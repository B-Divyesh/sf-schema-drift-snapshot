import { mkdir, copyFile } from 'node:fs/promises';
import path from 'node:path';

await mkdir(path.resolve('dist/bin'), { recursive: true });
await copyFile(path.resolve('target/release/sds'), path.resolve('dist/bin/sds'));
