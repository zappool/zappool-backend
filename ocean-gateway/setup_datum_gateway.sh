#!/bin/bash

# Clone datum-gateway repo, apply patch

repo_name="datum_gateway"
src_dir="./"$repo_name
repo_path="https://github.com/OCEAN-xyz/"$repo_name
tag="v0.4.0beta"
patch_file="../hooks-v040.c4d57a4.patch"

if [ ! -d "$src_dir" ]; then
    echo "Cloning the $repo_name git repository..."
    mkdir $src_dir
    git clone $repo_path --branch $tag
else
    echo "The src dir already exists ($src_dir)"
fi

cd $src_dir
git reset --hard
git checkout $tag
git status

# Apply patch
ls $patch_file
git apply $patch_file
git status

# Build the projext
cmake . && make

ls -l datum_gateway
