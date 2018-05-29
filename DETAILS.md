# In-Depth Explanation

This document is intended to provide the software-level explanation behind functionality and design decisions described in the README.

## TOML

TOML was chosen for its extreme simplicity in writing. It is free of much of the potential for syntax errors common in JSON documents, and allows for the concatenation of multiple update files to form a single larger update (such as migrating changes between channels), assuming there are no duplicate entries.
