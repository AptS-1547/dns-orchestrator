/**
 * 同步 package.json 的版本号到 tauri.conf.json 和 Cargo.toml
 */

import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

// 读取 package.json 版本
const packageJson = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf-8'));
const version = packageJson.version;

console.log(`同步版本号: ${version}`);

// 更新 tauri.conf.json
const tauriConfPath = resolve(root, 'dns-orchestrator-tauri/tauri.conf.json');
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'));
tauriConf.version = version;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
console.log('✓ dns-orchestrator-tauri/tauri.conf.json');

// 更新 Tauri Cargo.toml
const cargoTomlPath = resolve(root, 'dns-orchestrator-tauri/Cargo.toml');
let tauriCargoToml = readFileSync(cargoTomlPath, 'utf-8');
tauriCargoToml = tauriCargoToml.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
writeFileSync(cargoTomlPath, tauriCargoToml);
console.log('✓ dns-orchestrator-tauri/Cargo.toml');

// 更新 Actix-web Cargo.toml
const actixCargoTomlPath = resolve(root, 'dns-orchestrator-web/Cargo.toml');
let actixCargoToml = readFileSync(actixCargoTomlPath, 'utf-8');
actixCargoToml = actixCargoToml.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
writeFileSync(actixCargoTomlPath, actixCargoToml);
console.log('✓ dns-orchestrator-web/Cargo.toml');

console.log('版本同步完成');
