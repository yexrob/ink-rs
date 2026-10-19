1. Unsafe code is forbidden . If it's nessesary, request permission to use it.
2. Don't write junk code to cater to the test, and do a good job of architecture design.
3. Agents may commit in small steps without asking. Each commit holds one logical change that builds and passes the tests covering it, with a Conventional Commits message. Stage your own files by path, since other agents share the working tree. Pushing needs the user's approval.
