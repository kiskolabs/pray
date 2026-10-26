import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  buildLockfile,
  parseManifest,
  renderProject,
  resolveProject,
  serializeLockfile,
} from "../dist/index.js";
import { sha256Prefixed } from "../dist/hashing.js";

const ITERATIONS = Number(process.env.PRAY_BENCH_ITERATIONS ?? "500");
const WARMUP = Number(process.env.PRAY_BENCH_WARMUP ?? "20");
const PACKAGE_COUNT = Number(process.env.PRAY_BENCH_PACKAGES ?? "1");

if (!Number.isInteger(ITERATIONS) || ITERATIONS < 1) throw new Error("iterations must be positive");
if (!Number.isInteger(WARMUP) || WARMUP < 0) throw new Error("warmup must not be negative");
if (!Number.isInteger(PACKAGE_COUNT) || PACKAGE_COUNT < 1) throw new Error("packages must be positive");

function packageIdentity(index) {
  if (index === 0) return ["sample/base", "packages/base"];
  const suffix = index.toString().padStart(4, "0");
  return [`sample/pkg-${suffix}`, `packages/pkg-${suffix}`];
}

function buildManifest() {
  const lines = ['prayfile "1"', 'compose "A.md" do'];
  for (let index = 0; index < PACKAGE_COUNT; index += 1) {
    const [name, path] = packageIdentity(index);
    lines.push(`  pray "${name}", path: "${path}"`);
  }
  return `${lines.join("\n")}\nend\n`;
}

function buildPrayspec(name) {
  return `Package::Specification.new do |spec|
  spec.name = "${name}"
  spec.version = "1.0.0"
  spec.files = ["exports/a.md"]
  spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }
end
`;
}

const MANIFEST = buildManifest();
const PRAYSPECS = Array.from({ length: PACKAGE_COUNT }, (_, index) =>
  buildPrayspec(packageIdentity(index)[0]),
);

function summarize(samples) {
  samples.sort((left, right) => left - right);
  const middle = Math.floor(ITERATIONS / 2);
  return {
    p50:
      ITERATIONS % 2 === 0
        ? (samples[middle - 1] + samples[middle]) / 2
        : samples[middle],
    p95: samples[Math.ceil(ITERATIONS * 0.95) - 1],
  };
}

function measure(operation) {
  for (let index = 0; index < WARMUP; index += 1) operation();
  const samples = [];
  for (let index = 0; index < ITERATIONS; index += 1) {
    const started = process.hrtime.bigint();
    operation();
    samples.push(Number(process.hrtime.bigint() - started) / 1000);
  }
  return summarize(samples);
}

async function measureAsync(operation) {
  for (let index = 0; index < WARMUP; index += 1) await operation();
  const samples = [];
  for (let index = 0; index < ITERATIONS; index += 1) {
    const started = process.hrtime.bigint();
    await operation();
    samples.push(Number(process.hrtime.bigint() - started) / 1000);
  }
  return summarize(samples);
}

function printMeasurement(name, measurement) {
  console.log(
    `${name} p50_us=${measurement.p50.toFixed(2)} p95_us=${measurement.p95.toFixed(2)} ops_s=${(1_000_000 / measurement.p50).toFixed(0)}`,
  );
}

function serializeLock(project, rendered) {
  return serializeLockfile(
    buildLockfile({
      manifestHash: project.manifestHash,
      ...(project.environment ? { environment: project.environment } : {}),
      projectRoot: project.projectRoot,
      manifestSources: project.manifest.sources,
      manifestTargets: project.manifest.targets,
      rendered,
      packages: project.packages,
      sourceRevisions: project.sourceRevisions,
      sourceHostKeys: project.sourceHostKeys,
    }),
  );
}

const root = mkdtempSync(join(tmpdir(), "pray-compat-bench-"));
try {
  const manifestPath = join(root, "Prayfile");
  writeFileSync(manifestPath, MANIFEST);
  for (let index = 0; index < PACKAGE_COUNT; index += 1) {
    const [, path] = packageIdentity(index);
    mkdirSync(join(root, path, "exports"), { recursive: true });
    writeFileSync(join(root, path, "package.prayspec"), PRAYSPECS[index]);
    writeFileSync(join(root, path, "exports/a.md"), "A\n");
  }
  const project = await resolveProject(manifestPath);
  const rendered = renderProject(project);
  const benchmarkStarted = process.hrtime.bigint();
  const cpuStarted = process.cpuUsage();
  printMeasurement("parse", measure(() => parseManifest(MANIFEST)));
  printMeasurement("resolve", await measureAsync(() => resolveProject(manifestPath)));
  printMeasurement("render", measure(() => renderProject(project)));
  printMeasurement("lock", measure(() => serializeLock(project, rendered)));
  const renderedBytes = rendered.map((target) => target.content).join("");
  const inputBytes =
    Buffer.byteLength(MANIFEST) +
    PRAYSPECS.reduce((sum, spec) => sum + Buffer.byteLength(spec), 0) +
    2 * PACKAGE_COUNT;
  console.log(
    `fixture packages=${PACKAGE_COUNT} input_bytes=${inputBytes} output_bytes=${Buffer.byteLength(renderedBytes)} output_sha256=${sha256Prefixed(renderedBytes)}`,
  );
  const cpu = process.cpuUsage(cpuStarted);
  const wallMilliseconds = Number(process.hrtime.bigint() - benchmarkStarted) / 1_000_000;
  console.log(
    `resources wall_ms=${wallMilliseconds.toFixed(2)} cpu_ms=${((cpu.user + cpu.system) / 1000).toFixed(2)} peak_rss_bytes=${process.resourceUsage().maxRSS * 1024} iterations=${ITERATIONS} warmup=${WARMUP}`,
  );
} finally {
  rmSync(root, { recursive: true, force: true });
}
