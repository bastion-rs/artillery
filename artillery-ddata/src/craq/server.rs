use super::proto::*;
use super::{chain::CraqChain, erwlock::ERwLock, node::CRMode};
use crate::craq::node::CraqClient;

use core::sync::atomic::AtomicI64;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::sync::{atomic::Ordering, Arc};
use t1ha::T1haHashMap as HashMap;

pub struct CraqProtoServer {
    /// Tail connection pool receiver.
    pub tp_rx: Option<Arc<Receiver<CraqClient>>>,
    /// Successor connection pool receiver.
    pub sp_rx: Option<Arc<Receiver<CraqClient>>>,
    /// Tail connection pool sender.
    pub tp_tx: Option<Arc<Sender<CraqClient>>>,
    /// Successor connection pool sender.
    pub sp_tx: Option<Arc<Sender<CraqClient>>>,

    ///
    /// CR reference
    chain_ref: Arc<CraqChain>,
    /// Known objects: version, bytes
    pub objects: Arc<ERwLock<HashMap<i64, CraqObject>>>,
    /// Latest known version which is either clean or dirty.
    ///
    /// NOTE: This should start with a negative round to make
    /// version aligned at the first WR/DR.
    pub latest_version: AtomicI64,

    /// Latest clean version this one is always clean.
    ///
    /// NOTE: This should start with a negative round to make
    /// version aligned at the first clean version upgrade commit.
    pub latest_clean_version: AtomicI64,

    ///
    /// Algorithmic mode of operation
    cr_mode: CRMode,
}

impl CraqProtoServer {
    pub fn new(
        tp_rx: Option<Arc<Receiver<CraqClient>>>,
        tp_tx: Option<Arc<Sender<CraqClient>>>,
        sp_rx: Option<Arc<Receiver<CraqClient>>>,
        sp_tx: Option<Arc<Sender<CraqClient>>>,
        chain_ref: Arc<CraqChain>,
        cr_mode: CRMode,
    ) -> Self {
        Self {
            tp_rx,
            sp_rx,
            tp_tx,
            sp_tx,
            chain_ref,
            cr_mode,
            objects: Arc::new(ERwLock::new(HashMap::default())),
            latest_version: AtomicI64::new(!0),
            latest_clean_version: AtomicI64::new(!0),
        }
    }

    ///
    /// Creates shallow dirty copy of given object
    fn copy_object(&self, obj: &CraqObject, dirty: bool) -> CraqObject {
        let mut copied = obj.clone();
        copied.dirty = Some(dirty);
        copied
    }

    ///
    /// Handle all incoming write-mode requests like: test-and-set, write
    fn write(
        &self,
        obj: CraqObject,
        expected_version: i64,
    ) -> std::result::Result<i64, thrift::Error> {
        if self.tp_tx.is_none()
            || self.tp_rx.is_none()
            || self.sp_tx.is_none()
            || self.sp_rx.is_none()
        {
            return Err(state_error("Chain is not initialized!"));
        }

        if !self.chain_ref.is_head() {
            return Err(state_error("Cannot write to non-head!"));
        }

        if expected_version != !0 {
            // reject if latest version is not the expected version or there are uncommitted writes
            let latest_clean_version = self.latest_clean_version.load(Ordering::SeqCst);
            let latest_version = self.latest_version.load(Ordering::SeqCst);

            if latest_clean_version != expected_version || latest_version != latest_clean_version {
                return Ok(!0);
            }
        }

        // Record new object version. Do the Harlem Shake...
        let mut new_version = self.latest_version.fetch_add(1, Ordering::SeqCst);
        new_version += 1;
        self.objects.write().insert(new_version, obj.clone());

        // Send down chain
        let s_rx = self.sp_rx.as_ref().unwrap();
        let s_tx = self.sp_tx.as_ref().unwrap();
        // TODO: tryize
        let mut successor = s_rx.try_recv().unwrap();
        successor.write_versioned(obj, new_version)?;
        // TODO: tryize
        let _ = s_tx.try_send(successor);

        // Update current clean version
        let old_clean_ver: i64 = {
            // TODO: It should be CAS.
            let loaded = self.latest_clean_version.load(Ordering::SeqCst);
            if loaded < new_version {
                self.latest_clean_version
                    .store(new_version, Ordering::SeqCst);
                new_version
            } else {
                loaded
            }
        };

        if new_version > old_clean_ver {
            self.remove_old_versions(self.latest_clean_version.load(Ordering::SeqCst));
        }

        Ok(new_version)
    }

    ///
    /// Strong consistency specific version query
    /// This one makes a version request to tail to get the appropriate object
    fn get_obj_from_version_query(&self) -> std::result::Result<CraqObject, thrift::Error> {
        // Send a version query
        let t_rx = self.tp_rx.as_ref().unwrap();
        let t_tx = self.tp_tx.as_ref().unwrap();
        // TODO: tryize
        let mut tail = t_rx.try_recv().unwrap();
        let tail_version = tail.version_query()?;
        // TODO: tryize
        let _ = t_tx.try_send(tail);

        // If no clean version is around then return an empty obj
        if tail_version < 0 {
            return Ok(CraqObject::default());
        }

        let mut obj = self.objects.read().get(&tail_version).cloned();
        if obj.is_none() {
            // newer version already committed (old one erased), return the latest clean version
            obj = self
                .objects
                .read()
                .get(&self.latest_clean_version.load(Ordering::SeqCst))
                .cloned();
        }

        obj.map_or(
            Err(state_error("Returning empty object after a version query!")),
            Result::Ok,
        )
    }

    ///
    /// Removes all object versions older than the latest clean one.
    fn remove_old_versions(&self, latest_clean_ver: i64) {
        let mut objects = self.objects.write();
        objects.retain(|k, _| k >= &latest_clean_ver);
    }
}

impl CraqServiceSyncHandler for CraqProtoServer {
    ///
    /// Handles versioned query request received from the protocol client
    fn handle_version_query(&self) -> std::result::Result<i64, thrift::Error> {
        debug!(
            "[Artillery CRAQ Node {}] Received version query...",
            self.chain_ref.get_index()
        );

        // only tail should receive version queries
        if !self.chain_ref.is_tail() {
            return Err(state_error(
                "Cannot make a version query to a non-tail node!",
            ));
        }

        Ok(self.latest_clean_version.load(Ordering::SeqCst))
    }

    ///
    /// Handles versioned write request received from the protocol client
    fn handle_write_versioned(
        &self,
        obj: CraqObject,
        version: i64,
    ) -> std::result::Result<(), thrift::Error> {
        debug!(
            "[Artillery CRAQ Node {}] Received write with version: {}",
            self.chain_ref.get_index(),
            version
        );

        // Head should not receive versioned writes
        if self.chain_ref.is_head() {
            return Err(state_error("Cannot make a versioned write to the head!"));
        }

        // Write latest object version
        self.objects.write().insert(version, obj.clone());

        // Update latest version if applicable
        if self.latest_version.load(Ordering::SeqCst) < version {
            self.latest_version.store(version, Ordering::SeqCst);
        }

        // Non-tail: send down chain
        if !self.chain_ref.is_tail() {
            let s_rx = self.sp_rx.as_ref().unwrap();
            let s_tx = self.sp_tx.as_ref().unwrap();
            // TODO: tryize
            let mut successor = s_rx.try_recv().unwrap();
            successor.write_versioned(obj, version)?;
            // TODO: tryize
            let _ = s_tx.try_send(successor);
        }

        // Mark this current version as CLEAN
        let old_clean_ver: i64 = {
            // TODO: CAS it should be.
            let loaded = self.latest_clean_version.load(Ordering::SeqCst);
            if loaded < version {
                self.latest_clean_version.store(version, Ordering::SeqCst);
                version
            } else {
                loaded
            }
        };

        if version > old_clean_ver || self.chain_ref.is_tail() {
            self.remove_old_versions(self.latest_clean_version.load(Ordering::SeqCst));
        }

        Ok(())
    }

    ///
    /// Handles test-and-set request received from the protocol client
    fn handle_test_and_set(
        &self,
        obj: CraqObject,
        expected_version: i64,
    ) -> std::result::Result<i64, thrift::Error> {
        debug!(
            "[Artillery CRAQ Node {}] Received test-and-set request from client...",
            self.chain_ref.get_index()
        );

        self.write(obj, expected_version)
    }

    ///
    /// Handles write request received from the protocol client
    fn handle_write(&self, obj: CraqObject) -> std::result::Result<i64, thrift::Error> {
        debug!(
            "[Artillery CRAQ Node {}] Received write request from client...",
            self.chain_ref.get_index()
        );

        self.write(obj, !0)
    }

    fn handle_read(
        &self,
        model: CraqConsistencyModel,
        version_bound: i64,
    ) -> std::result::Result<CraqObject, thrift::Error> {
        debug!(
            "[Artillery CRAQ Node {}] Received read request from client...",
            self.chain_ref.get_index()
        );

        // Node hasn't initialized?
        if !self.chain_ref.is_tail()
            && (self.tp_rx.is_none()
                || self.tp_tx.is_none()
                || self.sp_rx.is_none()
                || self.sp_tx.is_none())
        {
            return Err(state_error("Chain is not initialized!"));
        }

        // Running normal CR: fail if we're not the tail
        if self.cr_mode == CRMode::Cr && !self.chain_ref.is_tail() {
            return Err(state_error("Cannot read from non-tail node in CR mode!"));
        }

        // No objects stored?
        if self.objects.read().is_empty() {
            return Ok(CraqObject::default());
        }

        // Lazy programmers do the best they say.
        // Same people said there's more than one way to do it.
        match model {
            CraqConsistencyModel::Strong => {
                if self.latest_version.load(Ordering::SeqCst)
                    > self.latest_clean_version.load(Ordering::SeqCst)
                    && !self.chain_ref.is_tail()
                {
                    // Non-tail: latest known version isn't clean, send a version query
                    let obj = self.get_obj_from_version_query()?;
                    return Ok(self.copy_object(&obj, true));
                }

                if self.latest_clean_version.load(Ordering::SeqCst) < 0 {
                    return Ok(CraqObject::default());
                }

                if self.chain_ref.is_tail() {
                    let latest_version = self.latest_version.load(Ordering::SeqCst);
                    if let Some(obj) = self.objects.read().get(&latest_version) {
                        return Ok(self.copy_object(obj, false));
                    } else {
                        return Err(state_error("Returning null object from the tail"));
                    }
                }

                let latest_clean_version = self.latest_clean_version.load(Ordering::SeqCst);
                if let Some(obj) = self.objects.read().get(&latest_clean_version) {
                    return Ok(self.copy_object(obj, false));
                } else {
                    return Err(state_error("Returning null object from a clean read!"));
                }
            }
            CraqConsistencyModel::Eventual => {
                // Return latest known version
                let latest_version = self.latest_version.load(Ordering::SeqCst);
                if let Some(obj) = self.objects.read().get(&latest_version) {
                    // TODO: normally None for dirty.
                    return Ok(self.copy_object(obj, false));
                } else {
                    return Err(state_error("Returning null object for an eventual read!"));
                }
            }
            CraqConsistencyModel::EventualMaxBounded => {
                // Return latest known version within the given bound
                let latest_version = self.latest_version.load(Ordering::SeqCst);
                let latest_clean_version = self.latest_clean_version.load(Ordering::SeqCst);
                let bounded_version = latest_clean_version.saturating_add(std::cmp::min(
                    version_bound,
                    latest_version.saturating_sub(latest_clean_version),
                ));
                if let Some(obj) = self.objects.read().get(&bounded_version) {
                    // TODO: normally None for dirty.
                    return Ok(self.copy_object(obj, false));
                } else {
                    return Err(state_error(
                        "Returning null object for a bounded eventual read!",
                    ));
                }
            }
            CraqConsistencyModel::Debug => {
                // make a version query
                if self.chain_ref.is_tail() {
                    return Ok(CraqObject::default());
                } else {
                    self.copy_object(&self.get_obj_from_version_query()?, true);
                }
            }
        }

        error!("Fatal state error happened.");
        Err(state_error("Fatal state error."))
    }
}

#[inline]
fn state_error(msg: &str) -> thrift::Error {
    thrift::Error::from(InvalidState::new(msg.to_owned()))
}
