# Contributing to TuxBridge

Thanks for helping! TuxBridge is made for people who have never used Linux, so the best
contributions keep it simple, friendly and explained in plain words.

## Reporting a problem or an idea

- **From the app:** the **Feedback** page (bottom left) sends it straight to the team, with an
  optional diagnostic report you can preview first. A failed task's **Report this** button fills
  it in for you.
- **On GitHub:** open an issue with what you did, what you expected, what happened, and your
  TuxBridge and Windows versions (Settings → About).

## Making a change

1. Set up the project as described in [docs/development.md](docs/development.md).
2. For anything larger than a small fix, open an issue first so we can agree on the approach.
3. Keep changes focused, and follow the style of the code around them.
4. Before opening a pull request:

   ```powershell
   pnpm exec tsc --noEmit
   cd src-tauri; cargo test --lib
   ```

5. If your change is visible to people, describe it in plain words in [CHANGELOG.md](CHANGELOG.md)
   and, where it helps, in the guides (`help/`).

## Writing for TuxBridge

- Write for someone new to Linux: plain words, short sentences, the app's own button names in
  **bold**.
- Guides, lessons and Pip's lines should be accurate. If you aren't sure a command or claim is
  right, check it on a real system first.
- The Security Basics challenge is strictly defensive: reading logs, verifying files, spotting
  mistakes. Contributions that teach attacking systems won't be accepted.

## Sign-off

Please sign off your commits (`git commit -s`), certifying the
[Developer Certificate of Origin](https://developercertificate.org): that you wrote the change,
or otherwise have the right to submit it under the project's license.

## License

By contributing, you agree that your contributions are licensed under the
[Apache License 2.0](LICENSE). "TuxBridge" and Pip remain trademarks of Rookery Software.
