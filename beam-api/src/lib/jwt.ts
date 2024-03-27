import * as jose from 'jose';
import { envs } from '@/env';
import { JWT_ALGORITHM as alg } from './constants';

// JWT Implementation notes:
// - Require token to specify algorithm was used
// - Prevent sidejacking by adding user context

// Authorization
// - Least privilege
// - Prefer ABAC and ReABAC over RBAC

// Parse keys from PEM file
const keyPemFile = Bun.file(envs.JWT_SECRET_PATH);
if (!(await keyPemFile.exists())) {
    throw new Error(`Private key file not found: ${keyPemFile}. Check JWT_SECRET_PATH environment variable.`);
}
const pemString = await keyPemFile.text();
const privateKeyRegex = /-----BEGIN PRIVATE KEY-----[\s\S]*?-----END PRIVATE KEY-----/;
const publicKeyRegex = /-----BEGIN PUBLIC KEY-----[\s\S]*?-----END PUBLIC KEY-----/;
const privateKeyMatch = pemString.match(privateKeyRegex);
const publicKeyMatch = pemString.match(publicKeyRegex);
const pkcs8 = privateKeyMatch ? privateKeyMatch[0] : null;
const spki = publicKeyMatch ? publicKeyMatch[0] : null;
if (!pkcs8) {
    throw new Error(`Private key not found in file: ${keyPemFile}`);
}
if (!spki) {
    throw new Error(`Public key not found in file: ${keyPemFile}`);
}

export const privateKey = await jose.importPKCS8(pkcs8, alg, {
    extractable: true,
})
export const publicKey = await jose.importSPKI(spki, alg, {
    extractable: true,
})
