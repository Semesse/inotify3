import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts'],
  target: 'es2015',
  splitting: false,
  sourcemap: true,
  dts: true,
  clean: true,
  external: [/.*.node$/],
})
