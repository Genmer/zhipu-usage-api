import { readFileSync, writeFileSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')

const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'))
const version = pkg.version

console.log(`[sync-version] ${version}`)

// tauri.conf.json
const tauriPath = resolve(root, 'src-tauri/tauri.conf.json')
const tauri = JSON.parse(readFileSync(tauriPath, 'utf8'))
tauri.version = version
writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n')

// Cargo.toml
const cargoPath = resolve(root, 'src-tauri/Cargo.toml')
const cargo = readFileSync(cargoPath, 'utf8')
writeFileSync(cargoPath, cargo.replace(/^version\s*=\s*".*"/m, `version = "${version}"`))

console.log('[sync-version] done')
