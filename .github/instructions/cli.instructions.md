---
applyTo: '**'
---

We are creating a cli called `meta-takeout` for interacting with zip files from Facebook and Instagram.

These files contain images and other documents from point-in-time exports from the external platform.

The exports that happen later in time sometimes have missing or different files from the exports that happened earlier, so we are taking great caution to ensure no data is lost.

We are using the crc32 hashes and the image similarity metrics to ensure we aren't writing duplicate files, ensuring we take the smallest/most compressed variant of the images provided to us by the external platform (we trust that the smaller size is due to more advanced compression, rather than lossy compression).


Here is how I imagine the general flow of the CLI working:

```pwsh
> meta-takeout config init # the CLI tool is cwd-agnostic
Enter the path to the destination directory where you want to store the decompressed files
> "C:\Users\TeamD\Downloads\facebookexport\dump" # strip quotes if the user provided a value wrapped in quotes
Enter the paths to the source directories containing the zip files, empty to continue:
> "C:\Users\TeamD\OneDrive\Documents\Backups\meta\facebook 2024-06"
> "C:\Users\TeamD\Downloads\facebookexport"
>
Enter the similarity threshold for images
Default (1) >
You chose: 1

Wonderful!
Found 0 files and folders in the destination path.
Found 27 zip files in the source paths.
> meta-takeout sync # this command will ensure that the contents on disk match the deduplicated content of the zip files, idempotent
Found 27 zip files in the source paths with 274835 files contained within. 
Found 0 files and folders in the destination path.
Identified 35000 unique names in the source zips.
Part 1: Of the 35000 uniquely named files, 13245 files (37.8%) are the same across all zips (crc32)
Part 2: Of the 35000 uniquely named files, 21000 files (62.1%) are images we will check perceptual similarity for. # The same name may be present in multiple zip files, we will assert all images are within similarity tolerance to one another
Part 3: Of the 35000 uniquely named files, 755 (2.2%) are documents with no diff support, each copy will be exported.
Part 4: Validation

===
part 1
===
Beginning sync of files. # If the file already exists in the destination, it will be skipped
Syncing file to disk, 13244 tasks remain (7.5MB/s, 32.5GB remain, 25m 34s ETA, 2 files per second)...
Syncing file to disk, 13243 tasks remain (7.2MB/s, 32.1GB remain, 25m 33s ETA, 3 files per second)...
Syncing file to disk, 13242 tasks remain (7.3MB/s, 31.6GB remain, 25m 30s ETA, 17 files per second)...
...
Done! Took 25m 18s to sync 13245 files to "C:\Users\TeamD\Downloads\facebookexport\dump"

===
part 2
===
Beginning sync of files, 21000 files have the same name (8921 unique names).
Found 3 files with the same name: "media\other\23967524_1338605629584566_3934457482159587328_n_17870308819197916.jpg"
Found the files in the following zip files:
- C:\Users\TeamD\OneDrive\Documents\Backups\meta\instagram-teamdman-2024-06-18-DveFYE6C.zip
- C:\Users\TeamD\OneDrive\Documents\Backups\meta\instagram-teamdman-2024-06-19-Ab12EECC.zip
- C:\Users\TeamD\OneDrive\Documents\Backups\meta\instagram-teamdman-2024-06-21-csabjk33.zip
Uncompressed size: min=22.7 kb, max=33.8 kb
Computing hashes for each image...
Image are perceptually similar ✅ (dist min=0, max=0.3, mean=0.1)
Syncing smallest file to disk (22.7 kb)
21997 files remain (3.5 MB/s 21.8 GB remain, 1h 23m 15s ETA, 0.1 files per second)...
Found 2 files with the same name: "some/other/image.png"
Found the files in the following zip files:
- ...
...
Done! Took 1h 16m 25s to identify and sync the best 8921 files.

===
part 3
===

Syncing files that we cannot diff.
Of the 755 files to sync, 25 (3.3%) already exist.
Syncing 730 files.
Syncing C:\Users\TeamD\OneDrive\Documents\Backups\meta\facebook 2024-06\facebook-Dominic9201-2024-06-19-228yS1FQ.zip\your_facebook_activity\messages\inbox\azeemisyour8thandfinalhungergameschampion_5791072597585718\videos\158506507_3633531676773709_4743268916235238963_n_893787944528185.mp4 to C:\Users\TeamD\Downloads\facebookexport\dump\your_facebook_activity\messages\inbox\azeemisyour8thandfinalhungergameschampion_5791072597585718\videos\facebook-Dominic9201-2024-06-19-228yS1FQ.zip\158506507_3633531676773709_4743268916235238963_n_893787944528185.mp4 # note that the .zip here is a directory name not a zip file name
729 files remain (9.1 MB/s, 16GB remain, 25m ETA, 2 files per second)...
Syncing {} to {}
728 files remain...

...
Done! Took 17m to sync 730 files.

===
part 4
===

Validating 13245+8921+730 files (22896 total).
✅ crc32 matched 22895 files remain (1.2 GB/s, 60.1 GB remain, 16m 22s ETA, 14 files per second)...
✅ crc32 matched 22894 files remain (1.1 GB/s, 60.0 GB remain, 16m 21s ETA, 16 files per second)...
...
Done! Took 1h 36m 22s to sync 27 zip files.