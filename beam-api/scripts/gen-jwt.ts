import { JWT_ALGORITHM as alg } from "@/lib/constants";
import { input } from "@inquirer/prompts";
import chalk from "chalk";
import * as jose from "jose";

console.log(`Generating keys using algorithm: ${alg}\n`);

// Generate a key pair
const { privateKey, publicKey } = await jose.generateKeyPair(alg, {
    extractable: true,
});

// Export the keys
const privateKeyPem = await jose.exportPKCS8(privateKey);
const publicKeyPem = await jose.exportSPKI(publicKey);

// Log the keys
// console.log(`Private key:\n${privateKeyPem}\n`);
// console.log(`Public key:\n${publicKeyPem}\n`);
console.log(`${chalk.underline.green("Private key:")}\n${privateKeyPem}\n`);
console.log(`${chalk.underline.green("Public key:")}\n${publicKeyPem}\n`);

// Prompt for file paths
// Default to `./jwt.pem`
const keyPath = await input({
    message: "Enter the path to save the key",
    default: "./jwt.pem",
    validate: (value) => {
        return value.endsWith(".pem") || "Path must end with .pem";
    },
});

// Write the keys to files
await Bun.write(keyPath, `${privateKeyPem}\n${publicKeyPem}`);
console.log(`Private and public keys saved to "${chalk.blue(keyPath)}"`);
