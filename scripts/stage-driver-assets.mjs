import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(scriptDir, '..');

const skipStage = ['1', 'true', 'yes'].includes(
  String(process.env.SKIP_DRIVER_STAGE ?? '').toLowerCase(),
);

if (skipStage) {
  console.log('[driver-stage] 已跳过驱动产物打包（SKIP_DRIVER_STAGE=1）');
  process.exit(0);
}

const sourceInput = process.env.DRIVER_ARTIFACTS_DIR || 'driver/windows/artifacts/driver';
const sourceDir = path.isAbsolute(sourceInput)
  ? sourceInput
  : path.resolve(projectRoot, sourceInput);

const targetDir = path.resolve(projectRoot, 'src-tauri/drivers/windows');
const manifestName = 'driver-package-manifest.json';
const allowedExts = new Set(['.sys', '.inf', '.cat']);

function fail(message) {
  console.error(`[driver-stage] ${message}`);
  process.exit(1);
}

if (!fs.existsSync(sourceDir) || !fs.statSync(sourceDir).isDirectory()) {
  fail(`未找到驱动产物目录：${sourceDir}`);
}

const sourceFiles = fs
  .readdirSync(sourceDir, { withFileTypes: true })
  .filter((entry) => entry.isFile() && allowedExts.has(path.extname(entry.name).toLowerCase()))
  .map((entry) => entry.name)
  .sort();

if (sourceFiles.length === 0) {
  fail(`目录中未找到可打包的驱动文件（.sys/.inf/.cat）：${sourceDir}`);
}

const missingKinds = [...allowedExts].filter(
  (ext) => !sourceFiles.some((name) => path.extname(name).toLowerCase() === ext),
);
if (missingKinds.length > 0) {
  fail(`驱动产物不完整，缺少：${missingKinds.join(', ')}`);
}

fs.mkdirSync(targetDir, { recursive: true });

for (const entry of fs.readdirSync(targetDir, { withFileTypes: true })) {
  if (!entry.isFile()) continue;

  const ext = path.extname(entry.name).toLowerCase();
  if (allowedExts.has(ext) || entry.name === manifestName) {
    fs.rmSync(path.join(targetDir, entry.name), { force: true });
  }
}

const copied = [];
for (const fileName of sourceFiles) {
  const sourcePath = path.join(sourceDir, fileName);
  const targetPath = path.join(targetDir, fileName);
  fs.copyFileSync(sourcePath, targetPath);

  copied.push({
    name: fileName,
    size: fs.statSync(targetPath).size,
  });
}

const manifest = {
  generated_at: new Date().toISOString(),
  source_dir: path.relative(projectRoot, sourceDir).replace(/\\/g, '/'),
  files: copied,
};

fs.writeFileSync(path.join(targetDir, manifestName), `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');

console.log('[driver-stage] 已完成驱动文件打包准备：');
for (const file of copied) {
  console.log(`  - ${file.name} (${file.size} bytes)`);
}
console.log(`[driver-stage] 清单文件：src-tauri/drivers/windows/${manifestName}`);
