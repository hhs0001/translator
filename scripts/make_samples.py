"""Generates sample subtitle files for manual UI testing.

Usage: python scripts/make_samples.py [out_dir]
"""

import os
import sys

SRT_LINES = [
    "Look, I told you this would happen.",
    "We don't have much time left.\nGet everyone to the roof.",
    "Are you seriously telling me that nobody checked the perimeter before we left?",
    "- Hey!\n- What?",
    "It was never about the money.",
]

ASS_SAMPLES = [
    r"{\an8\pos(640,120)\blur3}TOKYO - 07:42",
    r"Você {\i1}realmente{\i0} acha que isso vai funcionar?",
    r"{\fad(200,200)}I'm not going back there.\NNot after what happened.",
    r"{\k30}La{\k22}la{\k40}la, canta comigo",
    r"- Cuidado!\N- Eu vi, eu vi.",
]

ASS_HEADER = """[Script Info]
Title: Sample
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: sign,Arial,60,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,8,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"""


def timestamp(seconds: float, srt: bool = True) -> str:
    hours = int(seconds // 3600)
    minutes = int(seconds // 60) % 60
    secs = int(seconds) % 60
    if srt:
        return f"{hours:02d}:{minutes:02d}:{secs:02d},000"
    return f"{hours}:{minutes:02d}:{secs:02d}.00"


def write_srt(path: str, count: int, text_for) -> None:
    blocks = []
    for index in range(1, count + 1):
        start = timestamp(index * 4)
        end = timestamp(index * 4 + 3)
        blocks.append(f"{index}\n{start} --> {end}\n{text_for(index)}\n")
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(blocks))


def write_ass(path: str, count: int) -> None:
    events = []
    for index in range(1, count + 1):
        start = timestamp(index * 5, srt=False)
        end = timestamp(index * 5 + 4, srt=False)
        style = "sign" if index % 7 == 0 else "Default"
        text = ASS_SAMPLES[(index - 1) % len(ASS_SAMPLES)]
        events.append(f"Dialogue: 0,{start},{end},{style},,0,0,0,,{text}")
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(ASS_HEADER + "\n".join(events) + "\n")


def main() -> None:
    out_dir = sys.argv[1] if len(sys.argv) > 1 else "samples"
    os.makedirs(out_dir, exist_ok=True)

    write_srt(
        os.path.join(out_dir, "sample.srt"),
        60,
        lambda index: SRT_LINES[(index - 1) % len(SRT_LINES)],
    )
    write_ass(os.path.join(out_dir, "sample.ass"), 40)
    write_srt(
        os.path.join(out_dir, "big.srt"),
        2000,
        lambda index: (
            f"This is subtitle line number {index}, long enough to wrap in the "
            "editor and exercise text layout."
        ),
    )
    print("wrote samples to", out_dir)


if __name__ == "__main__":
    main()
