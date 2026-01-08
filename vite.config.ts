import { defineConfig } from 'vite';
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  server: {
    port: 3000
  },
  build: {
    minify: false
  },
  plugins: [wasmPack(['./rust-wasm-raytracer'])]
});
