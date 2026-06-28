# コード統計 (言語別サマリー)
stats:
    tokei src/ -t Rust

# ファイル別 LOC (大きい順)
stats-files:
    tokei src/ -t Rust --files

# 400 LOC 超えのファイルを検出 (超えたら exit 1)
stats-guard:
    #!/usr/bin/env python3
    import json, subprocess, sys
    out = subprocess.run(
        ["tokei", "src/", "-t", "Rust", "--output", "json"],
        capture_output=True, text=True, check=True,
    )
    reports = json.loads(out.stdout).get("Rust", {}).get("reports", [])
    bad = sorted(
        [(r["name"], r["stats"]["code"])
         for r in reports if r["stats"]["code"] > 400],
        key=lambda x: -x[1],
    )
    if bad:
        print("Files exceeding 400 LOC limit:")
        [print(f"  {loc:>4}  {name}") for name, loc in bad]
        sys.exit(1)
    print("All Rust files within 400 LOC limit.")

# 重複パターンスキャン (error あれば exit 1, warning は exit 0)
scan:
    ast-grep scan src/

# 警告もすべて error 扱いで厳格スキャン
scan-strict:
    ast-grep scan src/ --error

# CI ゲート: LOC ガード + 重複スキャン
check: stats-guard scan

# 開発レビュー: ファイル別統計 + 全スキャン
review: stats-files scan
