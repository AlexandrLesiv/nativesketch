const buffer = 'Ljava/nio/ByteBuffer;';
export const descriptors: Record<string, string> = {
  apiVersion: '()I',
  appNew: `(${buffer}J${buffer})I`,
  appCall: `(J${buffer}J${buffer})I`,
  appTakeWork: `(J${buffer}J${buffer})I`,
  workCopyInput: `(J${buffer}J${buffer})I`,
  workCall: `(J${buffer}JJ${buffer})I`,
  appComplete: `(J${buffer}JJ${buffer})I`,
  bufferLength: `(J${buffer})I`,
  bufferCopy: `(JJ${buffer}J${buffer})I`,
  bufferFree: `(J${buffer})I`,
  inputFree: `(J${buffer})I`,
  appFree: `(J${buffer})I`,
  resultRetireStopped: `(J${buffer})I`,
};
export function validateDescriptors(text: string): void {
  const entries = [...text.matchAll(/public native int (\w+)\([^\n]*\n\s+descriptor: (\S+)/g)].map(
    (match) => [match[1], match[2]],
  );
  const actual = Object.fromEntries(entries);
  if (
    entries.length !== 13 ||
    Object.keys(actual).length !== 13 ||
    Object.entries(descriptors).some(([key, value]) => actual[key] !== value)
  )
    throw new Error('Actual javap descriptors differ from frozen JNI contract');
}
export function validateExports(text: string, probe: boolean): void {
  const actual = new Set<string>();
  for (const line of text.split('\n')) {
    const name = /Java_dev_nativesketch_portable_\w+/.exec(line)?.[0];
    if (!name) continue;
    const defined =
      /^\s*[0-9a-f]+\s+[Tt]\s+_?Java_/i.test(line) ||
      /^\s*\d+:\s+[0-9a-f]+\s+\d+\s+FUNC\s+GLOBAL\s+DEFAULT\s+\d+\s+Java_/i.test(line);
    if (!defined) throw new Error('JNI symbol is not a defined function');
    actual.add(name);
  }
  const expected = new Set(
    Object.keys(descriptors).map((name) => `Java_dev_nativesketch_portable_PortableJniV1_${name}`),
  );
  if (probe)
    expected.add('Java_dev_nativesketch_portable_PendingExceptionProbe_callWithPendingException');
  if (actual.size !== expected.size || [...actual].some((name) => !expected.has(name)))
    throw new Error('Native export set does not match production/probe role');
}
export const expectedClasses: Record<string, string[]> = {
  portableJniHostTest: [
    'InputJniHostTest',
    'LifetimeJniHostTest',
    'PreflightJniHostTest',
    'PreparedJniHostTest',
    'WorkerJniHostTest',
  ],
  portableJniPendingExceptionTest: ['PendingExceptionJniHostTest'],
};
