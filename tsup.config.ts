import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts'],
  target: 'es2015',
  splitting: false,
  sourcemap: true,
  dts: true,
  clean: false, // don't remove *.node
  external: [/.*.node$/],
})
