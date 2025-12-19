# CollapseLoader: Security & Safety

### [Russian version](SECURITY-RU.md)

### Another security document [here](https://docs.google.com/document/d/1XNgaQqfSbuimnGh2WTufgm0pXGoeDI3ezNgcF6hPq9s/edit?usp=sharing)

CollapseLoader is a free tool for Minecraft cheat clients. We are 100% open and transparent to keep you safe.

Our safety architecture consists of four key pillars:

## 1. Everyone Can See Our Code

-   **Fully Open Source:** All the code for CollapseLoader is available on our [GitHub organization](https://github.com/CollapseLoader).
-   **No Secrets:** We have no hidden tricks, backdoors, or harmful code in the loader itself.
-   **Community Checks:** Anyone can audit our code to ensure it is safe and functions exactly as described.

## 2. We Check Clients for Viruses

We have a strict rule: **if a submitted client looks unsafe or contains malicious obfuscation, we do not add it.**

### Our Tool: CollapseScanner

We use a special tool called **CollapseScanner** to analyze files. It detects:

-   **Viruses:** Any malicious payloads or malware signatures.
-   **Bad Connections:** Attempts to connect to unauthorized IP addresses or data logging services (loggers).
-   **Malicious Obfuscation:** Code that is intentionally hidden to conceal harmful actions.

## 3. Warden: Our Integrity Protection

You may notice some clients use a system called **Warden**. This is our custom security layer designed to prevent the theft and unauthorized redistribution of exclusive clients.

### Why do we use Warden?

Developers spend a lot of time creating clients. Unfortunately, "client dumping" (stealing code) is common. Warden prevents this by:

-   **Encrypted Integrity:** Critical data within specific clients is encrypted and only decrypted in the computer's memory when the valid CollapseLoader session is active.
-   **Anti-Tamper:** It ensures that the client files haven't been modified or injected with third-party viruses before you run them.
-   **Session Verification:** Warden uses a native bridge to verify that the client was launched via the official CollapseLoader, preventing unauthorized launchers from running stolen copies.

**Is Warden safe?**
Yes. Unlike malicious obfuscation used by virus creators, Warden is **not** designed to hide malware. It is purely an anti JAR theft measure. Since the Loader itself is open source, you can see exactly how the Warden agent is initialized.

## 4. Safe Downloads & Builds

-   **Built Automatically:** We use **GitHub Actions** to compile the program. This means no human can secretly change the code during the build process.
-   **Transparent Chain:** The file you download matches our open-source code exactly.

---

## Safety Tips

Even with our checks, always be careful. Follow these tips:

-   **Official Sources:** Only download from our [GitHub releases page](https://github.com/CollapseLoader) or [collapseloader.org](https://collapseloader.org).
-   **Update Often:** Always use the latest version of the Loader to ensure you have the latest security definitions and Warden updates.

---

_Last updated: 17 December, 2025_
