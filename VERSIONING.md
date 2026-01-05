# Versioning Guide

This project follows [Semantic Versioning](https://semver.org/) (SemVer): `MAJOR.MINOR.PATCH`

## Version Bump Guidelines

### PATCH (0.3.1 → 0.3.2)

**When to use:** Bug fixes, performance improvements, documentation updates

**Examples:**

- ✅ Fixed PST index calculation bug
- ✅ Optimized thread pool configuration
- ✅ Updated README documentation
- ✅ Fixed typos or formatting
- ✅ Performance tweaks without API changes

**Command:**

```bash
python3 scripts/bump_version.py patch
```

---

### MINOR (0.3.1 → 0.4.0)

**When to use:** New features, non-breaking changes, significant improvements

**Examples:**

- ✅ Added new analysis tools (deep_analysis.py, visualize_boards.py)
- ✅ Added new command-line options
- ✅ New board setup types
- ✅ Enhanced self-play features
- ✅ New AI strength levels

**Command:**

```bash
python3 scripts/bump_version.py minor
```

---

### MAJOR (0.3.1 → 1.0.0)

**When to use:** Breaking changes, major refactors, incompatible API changes

**Examples:**

- ✅ Changed core game logic API
- ✅ Removed deprecated features
- ✅ Major architectural refactor
- ✅ Changed file formats (breaking compatibility)
- ✅ First stable release (0.x.x → 1.0.0)

**Command:**

```bash
python3 scripts/bump_version.py major
```

---

## Workflow

### Standard Workflow

```bash
# 1. Make your changes
# 2. Test thoroughly
# 3. Bump version
python3 scripts/bump_version.py [patch|minor|major]

# 4. Commit changes
git add Cargo.toml
git commit -m "chore: Bump version to X.Y.Z"
```

### Quick Workflow (with auto-commit)

```bash
# Bump version and create commit in one step
python3 scripts/bump_version.py patch --commit -m "fix: Description of fix"
```

### Dry Run (preview changes)

```bash
# See what would change without making changes
python3 scripts/bump_version.py patch --dry-run
```

---

## Recent Version History

### 0.3.2 (2026-01-06)

- Fixed critical PST index calculation bug (40% win rate imbalance)
- Optimized thread pool to use 6 threads on 8-core systems
- Reset ai_config.json to default values
- Added comprehensive analysis tools

### 0.3.1 (Previous)

- Self-play improvements
- Analysis enhancements

---

## Tips

1. **Always test before bumping:** Run `cargo build --release` and test your changes
2. **Use dry-run first:** Preview changes with `--dry-run` flag
3. **Write clear commit messages:** Explain what changed and why
4. **Update CHANGELOG:** Document significant changes in CHANGELOG.md (if exists)
5. **Tag releases:** Consider creating git tags for releases: `git tag v0.3.2`

---

## Questions?

- **Multiple changes?** Use the highest applicable bump type
- **Not sure?** When in doubt, use `patch` for small changes, `minor` for features
- **Breaking changes?** Always use `major` and document the breaking changes
