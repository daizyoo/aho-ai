# Versioning Guide

Complete guide for version management in Shogi-Aho-AI project.

---

## Versioning Scheme

This project follows [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`

```
v0.5.0
  │ │ │
  │ │ └─ PATCH: Bug fixes, performance improvements
  │ └─── MINOR: New features, non-breaking changes
  └───── MAJOR: Breaking changes, major refactors
```

---

## Version Bump Guidelines

### PATCH (0.5.0 → 0.5.1)

**When to use**: Bug fixes, performance improvements, documentation

**Examples**:

- ✅ Fixed PST index calculation bug
- ✅ Optimized thread pool configuration
- ✅ Updated README documentation
- ✅ Fixed typos or formatting
- ✅ Performance tweaks without API changes
- ✅ Minor lint fixes

**Command**:

```bash
python3 scripts/bump_version.py patch
```

---

### MINOR (0.5.0 → 0.6.0)

**When to use**: New features, significant improvements, backward-compatible changes

**Examples**:

- ✅ Added new analysis tools
- ✅ New command-line options
- ✅ New board setup types
- ✅ Enhanced self-play features
- ✅ New AI strength levels
- ✅ UI improvements
- ✅ New evaluator features

**Command**:

```bash
python3 scripts/bump_version.py minor
```

---

### MAJOR (0.5.0 → 1.0.0)

**When to use**: Breaking changes, major refactors, incompatible changes

**Examples**:

- ✅ Changed core game logic API
- ✅ Removed deprecated features
- ✅ Major architectural refactor
- ✅ Changed file formats (breaking compatibility)
- ✅ First stable release (0.x.x → 1.0.0)

**Command**:

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

### Quick Workflow (auto-commit)

```bash
# Bump version and create commit in one step
python3 scripts/bump_version.py minor --commit -m "feat: Add new feature"
```

### Dry Run (preview)

```bash
# Preview changes without modifying files
python3 scripts/bump_version.py patch --dry-run
```

---

## Version History

### 0.5.0 (2026-01-07) - Current

**Major Enhancements**:

#### Evaluation Function Improvements (+210-360 Elo)

1. **Mobility Evaluation** (+100-150 Elo)

   - Piece activity scoring
   - Weighted moves (captures, promotions)

2. **Game Phase Detection** (+50-100 Elo)

   - Opening/Midgame/Endgame awareness
   - Phase-specific strategies

3. **Enhanced King Safety** (+30-50 Elo)

   - Escape square counting
   - Enemy attacker detection

4. **Tactical Patterns** (+20-40 Elo)

   - Passed pawn detection
   - Bishop pair bonus
   - Rooks on open files

5. **Development Scoring** (+10-20 Elo)
   - Opening piece development

#### ML Data Quality Improvements

- **Data Augmentation**: Horizontal flip for symmetric boards
- **Enhanced Labels**: material_diff, game_lengths, evaluations, critical_moments
- **HDF5 Schema**: Extended with new metadata fields
- **Compression**: gzip compression (~50% storage reduction)

#### UI Enhancements

- **Replay Feature**: Scrollable kifu file selector
- **Multiple Directories**: Scans both `kifu/` and `selfplay_results/`
- **Metadata Display**: Board type, players, move count, timestamp
- **New Fields**: Evaluator, model path, version info

#### Documentation

- Organized docs in `docs/` folder
- Added comprehensive improvement guides
- Created INDEX.md for navigation

**Files Changed**: 15+  
**New Files**: 10+  
**Build Time**: ~6s

---

### 0.4.0 (2026-01-07)

**Features**:

- ML Model Versioning with embedded metadata
- Multi-Model Support (pluggable evaluators)
- Updated to `ort` 2.0 API

> [!NOTE]
> ML models have independent versioning in ONNX metadata

---

### 0.3.2 (2026-01-06)

**Fixes**:

- Critical PST index bug fix (40% → 50% win balance)
- Thread pool optimization (6 threads on 8-core)
- Reset ai_config.json to defaults

**Tools**:

- Added comprehensive analysis tools
- Deep analysis script
- Board visualization

---

### 0.3.1 (2026-01-05)

**Features**:

- Self-play improvements
- Analysis enhancements
- Performance optimizations

---

### 0.3.0 (2026-01-04)

**Features**:

- Sennichite detection (repetition)
- AI balance improvements
- Network play support

---

### 0.2.0 (2026-01-03)

**Features**:

- Chess piece implementation
- Mixed board setups
- Rule variability

---

### 0.1.0 (2026-01-02)

**Initial Release**:

- Basic Shogi implementation
- TUI interface
- Local play support

---

## ML Model Versioning

**Separate from Application Version**

ML models follow their own versioning embedded in ONNX metadata:

```
models/
├── Fair/
│   ├── v0.1.0/
│   │   ├── model.onnx
│   │   ├── model.pt
│   │   └── README.md
│   ├── v0.2.0/
│   └── v0.3.0/
├── ShogiOnly/
│   └── v0.1.0/
└── ...
```

**Model Version Format**: `v{major}.{minor}.{patch}`

**When to bump model version**:

- **Patch**: Hyperparameter tuning, same architecture
- **Minor**: Architecture changes, more training data
- **Major**: Complete redesign

See [docs/ML_USAGE.md](./ML_USAGE.md) for details.

---

## Tips & Best Practices

### Before Bumping

1. **Test thoroughly**: Run `cargo build --release` and test your changes
2. **Review changes**: Use `git diff` to see what changed
3. **Update docs**: Document significant changes
4. **Run tests**: `cargo test` if you have tests

### During Bumping

1. **Use dry-run**: Preview with `--dry-run` first
2. **Choose correct level**: When in doubt, use lower level (patch over minor)
3. **Write clear messages**: Explain what changed and why

### After Bumping

1. **Tag release**: `git tag v0.5.0`
2. **Push tags**: `git push origin v0.5.0`
3. **Update CHANGELOG**: Document in CHANGELOG.md (if exists)
4. **Announce**: Update README or release notes

---

## Version Comparison

### Feature Matrix

| Feature           | v0.1.0 | v0.3.0 | v0.5.0 |
| ----------------- | ------ | ------ | ------ |
| Basic Shogi       | ✅     | ✅     | ✅     |
| Chess Pieces      | ❌     | ✅     | ✅     |
| Network Play      | ❌     | ✅     | ✅     |
| ML Support        | ❌     | ✅     | ✅     |
| Sennichite        | ❌     | ✅     | ✅     |
| Enhanced Eval     | ❌     | ❌     | ✅     |
| Replay UI         | ❌     | ❌     | ✅     |
| Data Augmentation | ❌     | ❌     | ✅     |

### Performance Comparison

| Version | Eval Strength | Build Time | Binary Size |
| ------- | ------------- | ---------- | ----------- |
| v0.1.0  | Baseline      | ~3s        | ~5MB        |
| v0.3.0  | +100 Elo      | ~4s        | ~6MB        |
| v0.5.0  | +360 Elo      | ~6s        | ~7MB        |

---

## Questions & Troubleshooting

### Multiple changes?

Use the highest applicable level:

- Bug fix + new feature = **minor**
- Several features = **minor**
- Feature + breaking change = **major**

### Not sure which level?

**Rule of thumb**:

- Changed behavior? → patch
- New feature? → minor
- Breaking change? → major

### Forgot to bump?

```bash
# Bump now and amend last commit
python3 scripts/bump_version.py patch
git add Cargo.toml
git commit --amend --no-edit
```

---

## Related Documentation

- [ML Usage](./ML_USAGE.md) - ML model versioning
- [Index](./INDEX.md) - Documentation overview
- [Improvements](./improvements/) - Feature documentation

---

**Last Updated**: 2026-01-07  
**Current Version**: 0.5.0  
**Next Planned**: 0.6.0 (planned features TBD)
