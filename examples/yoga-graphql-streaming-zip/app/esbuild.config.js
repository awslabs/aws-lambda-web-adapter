import * as esbuild from 'esbuild';
import { mkdir, writeFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function build() {
  // Create dist directory if it doesn't exist
  await mkdir('dist', { recursive: true });

  // Build the project using esbuild
  const result = await esbuild.build({
    entryPoints: ['server.ts'],
    bundle: true,
    minify: false,
    platform: 'node',
    target: 'node22',
    outfile: 'dist/server.js',
    format: 'cjs',
    mainFields: ['module', 'main'],
    external: [
      "node:*"
    ],
    metafile: true,
  });

  // Create a minimal package.json for the dist folder
  const minimalPackageJson = {
    "name": "graphql-yoga",
    "version": "1.0.0",
    "type": "commonjs",
    "dependencies": {}
  };

  await writeFile(
    join(__dirname, 'dist', 'package.json'),
    JSON.stringify(minimalPackageJson, null, 2)
  );

  console.log('Build complete!');
  console.log(`Bundle size: ${Math.round(result.metafile.outputs['dist/server.js'].bytes / 1024)}KB`);
}

build().catch(err => {
  console.error('Build failed:', err);
  process.exit(1);
});
