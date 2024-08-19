#!/bin/bash

cd target/x86_64-unknown-linux-musl/release/

scp -i ~/authorized_keys_mes tfzc_aps tfzc_aps_service tfzc_iot_service tfzc_scada tfzc_scada_service tfzc_sync tfzc_sync_service tfzc_stats root@34.150.61.92:/root/

sleep 3

rm -f tfzc_aps tfzc_aps_service tfzc_iot_service tfzc_scada tfzc_scada_service tfzc_sync tfzc_sync_service tfzc_stats

cd ../../../