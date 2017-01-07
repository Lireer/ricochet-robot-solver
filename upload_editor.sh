#!/bin/bash

# Pull requests and commits to other branches shouldn't try to deploy, just build to verify
if [ "$TRAVIS_PULL_REQUEST" != "false" ] || [ "$TRAVIS_BRANCH" != "master" ]; then
    echo "Only updating github pages on commits on the master branch"
    exit 0
fi

rm -rf out || exit 0;
mkdir out;

GH_REPO="@github.com/Lireer/ricochet-robot-solver.git"

FULL_REPO="https://$GH_TOKEN$GH_REPO"

cd out
git init
git config user.name "travis"
git config user.email "travis@travis.invalid"
cp ../index.html index.html

git add .
git commit -m "deployed to github pages"
git push --force --quiet $FULL_REPO master:gh-pages
