import { access } from 'node:fs/promises';
import { delimiter, join } from 'node:path';
import type { ActionSpec } from './contracts/index.ts';
interface Definition {
  id: string;
  label: string;
  category: string;
  platform: string;
  args: string[];
  tools: string[];
  host?: string;
  foreground?: boolean;
}
const definitions: Definition[] = [
  ...[
    'macos',
    'ios',
    'visionos',
    'tvos',
    'watchos',
    'android',
    'linux',
    'windows',
    'web',
    'wasm',
  ].map((platform) => ({
    id: `build-${platform}`,
    label: `Build ${platform}`,
    category: 'Build',
    platform,
    args: ['scripts/build.ts', '--platform', platform],
    tools:
      platform === 'android'
        ? ['cargo', 'gradle', 'java']
        : platform === 'windows'
          ? ['cargo', 'dotnet']
          : ['cargo'],
    ...(platform === 'macos' || ['ios', 'visionos', 'tvos', 'watchos'].includes(platform)
      ? { host: 'darwin' }
      : {}),
  })),
  {
    id: 'build-macos-release',
    label: 'Build macOS release',
    category: 'Build',
    platform: 'macos',
    args: ['scripts/build.ts', '--platform', 'macos', '--profile', 'release'],
    tools: ['cargo', 'swift'],
    host: 'darwin',
  },
  {
    id: 'check-core',
    label: 'Background core checks',
    category: 'Check',
    platform: 'portable',
    args: ['scripts/check.ts', '--core'],
    tools: ['cargo'],
  },
  {
    id: 'check-web',
    label: 'Web build and adapter checks',
    category: 'Check',
    platform: 'web',
    args: ['scripts/check.ts', '--platform', 'web'],
    tools: ['cargo'],
  },
  {
    id: 'prepare-macos',
    label: 'Prepare macOS native tests',
    category: 'Prepare',
    platform: 'macos',
    args: ['scripts/test-e2e.ts', '--platform', 'macos', '--prepare-only'],
    tools: ['cargo', 'xcodebuild'],
    host: 'darwin',
  },
  {
    id: 'preview-web',
    label: 'Build isolated Wasm preview',
    category: 'Preview',
    platform: 'web',
    args: ['scripts/build.ts', '--platform', 'web'],
    tools: ['cargo'],
  },
  {
    id: 'export-report',
    label: 'Export offline report',
    category: 'Export',
    platform: 'portable',
    args: ['scripts/workbench.ts', 'export'],
    tools: [],
  },
  {
    id: 'run-macos',
    label: 'Run macOS app',
    category: 'Run',
    platform: 'macos',
    args: ['scripts/run.ts', '--platform', 'macos'],
    tools: [],
    foreground: true,
  },
  {
    id: 'native-tests',
    label: 'Execute native tests',
    category: 'Native tests',
    platform: 'macos',
    args: ['scripts/test-e2e.ts', '--platform', 'macos', '--interactive'],
    tools: [],
    foreground: true,
  },
];
async function available(tool: string) {
  for (const path of (process.env.PATH ?? '').split(delimiter)) {
    try {
      await access(join(path, tool));
      return true;
    } catch {
      /* Try next PATH directory. */
    }
  }
  return false;
}
export async function actionCatalog(): Promise<ActionSpec[]> {
  return Promise.all(
    definitions.map(async (item) => {
      const missing = (
        await Promise.all(item.tools.map(async (tool) => ((await available(tool)) ? null : tool)))
      ).filter(Boolean);
      const disabledReason = item.foreground
        ? 'Foreground execution is disabled by service policy.'
        : process.platform === 'win32' || item.platform === 'windows'
          ? 'Windows process-tree cleanup requires native-host verification.'
          : item.host && item.host !== process.platform
            ? `Requires ${item.host} host.`
            : missing.length
              ? `Missing tools: ${missing.join(', ')}.`
              : process.versions.node !== '24.22.0'
                ? 'Requires Node 24.22.0.'
                : null;
      return {
        id: item.id,
        label: item.label,
        category: item.category,
        platform: item.platform,
        commandPreview: `node ${item.args.join(' ')}`,
        foreground: !!item.foreground,
        enabled: !disabledReason,
        disabledReason,
        prerequisites: item.tools,
      };
    }),
  );
}
export function actionArguments(id: string, artifactPath: string) {
  const item = definitions.find((entry) => entry.id === id);
  if (!item || item.foreground) throw new Error('Unknown or foreground action');
  return [
    ...item.args,
    ...(id === 'export-report'
      ? ['--output', `${artifactPath}/report.html`]
      : id === 'prepare-macos'
        ? ['--output', `${artifactPath}/native`]
        : []),
  ];
}
