/** Emits runner events; stdout from tests is deliberately not interpreted as test records.
 * @param {AsyncIterable<{type:string,data:unknown}>} source */
export default async function* reporter(source) {
  for await (const event of source) {
    if (['test:pass', 'test:fail', 'test:summary'].includes(event.type)) {
      yield `${JSON.stringify(event)}\n`;
    }
  }
}
