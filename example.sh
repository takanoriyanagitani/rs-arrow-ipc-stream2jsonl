#!/bin/sh

ifile=./sample.d/tmp.file.ipc
istrm=./sample.d/tmp.strm.ipc

gencsv(){
	echo timestamp,severity,status,body
	echo 2025-10-21T01:41:32.012345Z,INFO,200,apt update done
	echo 2025-10-20T01:41:32.012345Z,WARN,500,apt update failure
	echo 2025-10-19T01:41:32.012345Z,WARN,500,apt update failure
}

geninput(){
	echo generating input file...

	mkdir -p ./sample.d

	gencsv |
		csv2arrow2ipc |
		cat > "${ifile}"

	arrow-file-to-stream "${ifile}" > "${istrm}"
}

test -f "${istrm}" || geninput

echo example 1: converting the sample ipc stream...
cat "${istrm}" |
	./arrow-ipc-stream2jsonl |
	jq -c

which dirents2arrow-ipc-stream |
	fgrep -q dirents2arrow-ipc-stream || exec sh -c '
		echo 'dirents2arrow-ipc-stream missing.'
		exit 1
	'

echo
echo example 2: converting the dirent ipc stream...

dirents2arrow-ipc-stream . |
	./arrow-ipc-stream2jsonl |
	jq -c '{
		basename,
		filetype,
		path,
		len,
		modified,
	}'
