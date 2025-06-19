# Thrumzip

<p align="center">
  <img src="logo.png" alt="Thrumzip Logo" width="200"/>
</p>

You can export your data from Meta, giving you a bunch of zip files to work with.

If you perform this export multiple times, that means you have even more zip files.

Do the newer files actually contain all the information in the older files?

Who knows!

Here's some data

```
Found 29 zip files
Stats by extension:
jpg: count=39677 | CRC(matches=253766 mismatches=10440 zeros=0) | SIZE(>=46 <=10394 ==253766)
png: count=17455 | CRC(matches=198885 mismatches=2 zeros=0) | SIZE(>=0 <=2 ==198885)
mp4: count=6634 | CRC(matches=17382 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==17382)
gif: count=3568 | CRC(matches=30930 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==30930)
json: count=717 | CRC(matches=25 mismatches=542 zeros=0) | SIZE(>=517 <=22 ==28)
aac: count=120 | CRC(matches=322 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==322)
pdf: count=110 | CRC(matches=2266 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==2266)
docx: count=30 | CRC(matches=231 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==231)
txt: count=30 | CRC(matches=14 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==14)
mp3: count=27 | CRC(matches=234 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==234)
wav: count=24 | CRC(matches=55 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==55)
zip: count=10 | CRC(matches=355 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==355)
mid: count=8 | CRC(matches=13 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==13)
webp: count=6 | CRC(matches=8 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==8)
sql: count=5 | CRC(matches=194 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==194)
py: count=4 | CRC(matches=6 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==6)
xlsx: count=4 | CRC(matches=4 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==4)
djvu: count=3 | CRC(matches=383 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==383)
java: count=2 | CRC(matches=2 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==2)
m4a: count=2 | CRC(matches=7 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==7)
flac: count=2 | CRC(matches=2 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==2)
jar: count=1 | CRC(matches=1 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==1)
s: count=1 | CRC(matches=190 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==190)
rtf: count=1 | CRC(matches=1 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==1)
heic: count=1 | CRC(matches=0 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==0)
ogg: count=1 | CRC(matches=1 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==1)
eml: count=1 | CRC(matches=1 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==1)
fasta: count=1 | CRC(matches=1 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==1)
qmbl: count=1 | CRC(matches=1 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==1)
cpp: count=1 | CRC(matches=0 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==0)
md: count=1 | CRC(matches=3 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==3)
pak: count=1 | CRC(matches=1 mismatches=0 zeros=0) | SIZE(>=0 <=0 ==1)

Validation summary:
  Checked entries: 29000
  Passes:          29000
  Failures:        0
```

Most jpg files remained the same size, many got smaller, and very few got bigger.
Presumably, Meta has compressed them better since the last export.

Thankfully, the CRC values in the zip file match when we compute the CRC ourselves.

Expectedly, the JSON files grow larger over time, though some remain the same or shrink.

---

Turns out that having `RUST_BACKTRACE="1"` causes `get_splat_path` to take a looot longer.