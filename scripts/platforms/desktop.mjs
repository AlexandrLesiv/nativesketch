import { mkdir, copyFile } from 'node:fs/promises';
import { join, delimiter } from 'node:path';
import { command, root, requireHost } from '../lib/context.mjs';
export async function buildLinux() {
  const env = { CARGO_TARGET_DIR: join(root, 'target/linux') };
  await command('cargo', ['build', '--locked', '--manifest-path', 'apps/linux/Cargo.toml'], {
    env,
  });
  await mkdir(join(root, 'artifacts/linux'), { recursive: true });
  await copyFile(
    join(root, 'target/linux/debug/native-sketch-gtk'),
    join(root, 'artifacts/linux/native-sketch-gtk'),
  );
}
export async function checkLinux() {
  const env = { CARGO_TARGET_DIR: join(root, 'target/linux') };
  await command('cargo', ['fmt', '--manifest-path', 'apps/linux/Cargo.toml', '--', '--check'], {
    env,
  });
  await command(
    'cargo',
    ['clippy', '--locked', '--manifest-path', 'apps/linux/Cargo.toml', '--', '-D', 'warnings'],
    { env },
  );
  await command('cargo', ['test', '--locked', '--manifest-path', 'apps/linux/Cargo.toml'], { env });
  await buildLinux();
}
export async function buildWindows() {
  requireHost('win32');
  await command('cargo', [
    'build',
    '--locked',
    '--release',
    '-p',
    'sketch-ffi',
    '--target',
    'x86_64-pc-windows-msvc',
  ]);
  await command('dotnet', ['build', 'apps/windows/NativeSketch.Windows.csproj', '-c', 'Release']);
}
export async function checkWindows() {
  await buildWindows();
  await command(
    'dotnet',
    ['run', '--project', 'apps/windows/InteropCheck', '-c', 'Release', '-r', 'win-x64'],
    {
      env: {
        PATH: `${join(root, 'target/x86_64-pc-windows-msvc/release')}${delimiter}${process.env.PATH ?? ''}`,
      },
    },
  );
}
