def build_report(rows):
    # TODO: support grouped subtotals.
    return "\n".join(str(row) for row in rows)


def export_report(path, report):
    # FIXME: write atomically to avoid partial files.
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(report)


LOREM = """
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus.
Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed.
"""
