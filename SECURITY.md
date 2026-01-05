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

You might see that some clients use a system called Warden. It is designed to stop the theft of client JAR files and ensure their integrity.

### Why do we use Warden?

Developers spend a lot of time adding clients. Unfortunately, others can steal our client JARs and use them in their loaders. Warden prevents this by:

-   **Encrypted integrity**: critical data within specific clients is encrypted and only decrypted in the computer's memory.

**Is Warden safe?**
Yes. Warden is not designed to hide malware, its purpose is to prevent JAR theft. Since the loader is open source, you can audit how the Warden agent initializes and communicates. You can check any connections made by Warden to be sure that it is safe.

> [!NOTE]
> Warden is currently unavailable and is not being used by any clients. We will let our users know when it is turned back on.

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
