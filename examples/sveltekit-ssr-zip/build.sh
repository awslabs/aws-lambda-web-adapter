#!/bin/bash

# remove the existing build directory created by @sveltejs/adapter-node
rm -rf ./app/build

# switch to the app directory
cd ./app

# install app deps and build
npm install
npm run build

# copy package.json to build directory
cp ./package.json ./build

# add custom wrapper to build directory
# https://docs.aws.amazon.com/lambda/latest/dg/runtimes-modify.html#runtime-wrapper
cat << EOF > ./build/run.sh
#!/bin/bash

node index.js
EOF