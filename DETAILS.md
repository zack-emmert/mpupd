# In-Depth Explanation

This document is intended to provide the software-level explanation behind functionality and design decisions described in the README.

## TOML

TOML was chosen for its extreme simplicity in writing. It is free of much of the potential for syntax errors common in JSON documents, and allows for the concatenation of multiple update files to form a single larger update (such as migrating changes between channels), assuming there are no duplicate entries.

## Channel Files

1. Update URLs must not change

    This requirement is due to the way the software logs the updates it has already completed. It does so by storing the update URL in a log file. If the update URL changes, it appears to the client that the update did not take place, causing the update to be repeated, which may result in unintended behavior.
