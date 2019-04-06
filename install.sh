#!/bin/bash
if [ -f "/usr/local/bin/rtsql" ]; then
	rm /usr/local/bin/rtsql
fi

if [ -f "/usr/local/etc/rtdownloader/config.json" ]; then
	rm /usr/local/etc/rtdownloader/config.json
fi

cp ./config.json /usr/local/etc/rtdownloader/

cargo build --release
cp ./target/release/rtsql /usr/local/bin/rtsql
echo "Successfully Installed."
exit 0
