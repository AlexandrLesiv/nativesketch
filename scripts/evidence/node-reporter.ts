/** Emits runner events; stdout from tests is deliberately not interpreted as test records. */
export default async function* reporter(
  source: AsyncIterable<{
    type: string;
    data: unknown;
  }>,
) {
  for await (const event of source) {
    if (['test:pass', 'test:fail', 'test:summary'].includes(event.type)) {
      yield `${JSON.stringify(event)}\n`;
    }
  }
}
