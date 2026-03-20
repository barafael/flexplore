#!/usr/bin/env node
// Renders each testdata/*/expected.html to testdata/*/rendered_html.png
// Usage: npx playwright test --config=... OR just: node scripts/render_html.mjs [case1 case2 ...]

import { chromium } from 'playwright';
import { readdir, access } from 'fs/promises';
import { resolve, join } from 'path';
import { fileURLToPath } from 'url';
import { pathToFileURL } from 'url';

const __dirname = resolve(fileURLToPath(import.meta.url), '..');
const testdataDir = resolve(__dirname, '..', 'testdata');
const WIDTH = 800;
const HEIGHT = 600;

async function main() {
  const args = process.argv.slice(2);

  let cases;
  if (args.length > 0) {
    cases = args;
  } else {
    const entries = await readdir(testdataDir, { withFileTypes: true });
    cases = entries.filter(e => e.isDirectory()).map(e => e.name).sort();
  }

  console.error(`Will render ${cases.length} HTML test case(s)`);

  const browser = await chromium.launch();
  const context = await browser.newContext({
    viewport: { width: WIDTH, height: HEIGHT },
    deviceScaleFactor: 1,
  });

  for (const name of cases) {
    const htmlPath = join(testdataDir, name, 'expected.html');
    const outPath = join(testdataDir, name, 'rendered_html.png');

    try {
      await access(htmlPath);
    } catch {
      console.error(`  SKIP: ${htmlPath} not found`);
      continue;
    }

    const page = await context.newPage();
    await page.goto(pathToFileURL(htmlPath).href, { waitUntil: 'networkidle' });
    await page.screenshot({ path: outPath, fullPage: false });
    await page.close();
    console.error(`  Saved: ${outPath}`);
  }

  await browser.close();
  console.error('All done!');
}

main().catch(e => { console.error(e); process.exit(1); });
