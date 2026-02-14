import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const thisDir = dirname(fileURLToPath(import.meta.url));
const nodeDir = resolve(thisDir, "..");
const rootDir = resolve(nodeDir, "..");

const rootCargoTomlPath = resolve(rootDir, "Cargo.toml");
const nodeCargoTomlPath = resolve(nodeDir, "Cargo.toml");
const packageJsonPath = resolve(nodeDir, "package.json");
const packageLockPath = resolve(nodeDir, "package-lock.json");

function readPackageVersionFromCargoToml(content) {
  const lines = content.split(/\r?\n/);
  let inPackageSection = false;

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed === "[package]") {
      inPackageSection = true;
      continue;
    }
    if (inPackageSection && trimmed.startsWith("[")) {
      break;
    }
    if (inPackageSection) {
      const match = trimmed.match(/^version\s*=\s*"([^"]+)"\s*$/);
      if (match) {
        return match[1];
      }
    }
  }

  throw new Error("Could not find [package].version in Cargo.toml");
}

function updatePackageVersionInCargoToml(content, nextVersion) {
  const lines = content.split(/\r?\n/);
  let inPackageSection = false;
  let updated = false;

  for (let i = 0; i < lines.length; i += 1) {
    const trimmed = lines[i].trim();
    if (trimmed === "[package]") {
      inPackageSection = true;
      continue;
    }
    if (inPackageSection && trimmed.startsWith("[")) {
      break;
    }
    if (inPackageSection && /^\s*version\s*=/.test(lines[i])) {
      lines[i] = `version = "${nextVersion}"`;
      updated = true;
      break;
    }
  }

  if (!updated) {
    throw new Error("Could not update [package].version in node/Cargo.toml");
  }

  return `${lines.join("\n")}\n`;
}

const rootCargoToml = readFileSync(rootCargoTomlPath, "utf8");
const crateVersion = readPackageVersionFromCargoToml(rootCargoToml);

const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
packageJson.version = crateVersion;

if (packageJson.optionalDependencies) {
  for (const depName of Object.keys(packageJson.optionalDependencies)) {
    if (depName.startsWith(`${packageJson.name}-`)) {
      packageJson.optionalDependencies[depName] = crateVersion;
    }
  }
}

writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);

if (existsSync(packageLockPath)) {
  const packageLock = JSON.parse(readFileSync(packageLockPath, "utf8"));
  packageLock.version = crateVersion;
  if (packageLock.packages && packageLock.packages[""]) {
    packageLock.packages[""].version = crateVersion;
  }
  writeFileSync(packageLockPath, `${JSON.stringify(packageLock, null, 2)}\n`);
}

const nodeCargoToml = readFileSync(nodeCargoTomlPath, "utf8");
const updatedNodeCargoToml = updatePackageVersionInCargoToml(
  nodeCargoToml,
  crateVersion,
);
writeFileSync(nodeCargoTomlPath, updatedNodeCargoToml);

console.log(`Synced Node package versions to ${crateVersion}`);
