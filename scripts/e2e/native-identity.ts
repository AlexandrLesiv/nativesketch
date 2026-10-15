import type { Source } from '../evidence/types.ts';
import type { AppIdentity } from './subject-apps.ts';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { command } from '../lib/context.ts';
import { logged } from './xcode.ts';

export async function captureNativeIdentity(
  output: string,
  build: {
    source: Source;
    bundleIdentifier: string;
    binarySha256: string;
    testBinarySha256: string;
    focusApp: AppIdentity;
  },
) {
  const toolchain = await logged(output, 'toolchain.log', 'xcodebuild', ['-version']);
  const os = await command('sw_vers', [], { capture: true });
  const device = `${await command('sysctl', ['-n', 'hw.model'], { capture: true })}/${process.arch}`;
  await writeFile(
    join(output, 'identity.json'),
    JSON.stringify(
      {
        ...build,
        toolchain,
        os,
        device,
        runnerIdentityArtifact: 'runner-identity.json',
        inputRoute:
          'XCUIAutomation; gesture cases additionally use native CGEvent/AX; no physical pen',
        buildFeatures: [
          'native-test-controls (DEBUG-only; individual test arms disposable worker boundary)',
        ],
      },
      null,
      2,
    ),
  );
  return { toolchain, os, device };
}
