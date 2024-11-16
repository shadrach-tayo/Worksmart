cn release draft notor/worksmart 0.1.1
cn release draft  notor/worksmart 0.1.1 --channel beta

cn release upload notor/worksmart 01JCCS5MHTP73PYW1N1AYT7PSM --framework tauri

# convert one camera frame from the macos avfoundation backend as input to the specified filename
# start transcoding at -ss 0.5 and stop at -t 2
# -y overwrite output file
ffmpeg -y -ss 0.5 -t 2 -f avfoundation -framerate 30 -i "0" -vframes 1 filename.png
