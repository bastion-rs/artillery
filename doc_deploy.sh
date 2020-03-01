#!/usr/bin/env sh

yarn global add vuepress
yarn global add vuepress-theme-api

cd site/
vuepress build .
cd -

# Always get removed, sigh...
touch docs/.nojekyll
echo "artillery.bastion.rs" >> docs/CNAME

