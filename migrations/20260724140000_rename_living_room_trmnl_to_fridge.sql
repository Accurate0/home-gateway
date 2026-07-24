UPDATE eink_display SET device_id = 'fridge-trmnl', name = 'Fridge TRMNL' WHERE device_id = 'living-room-trmnl';

UPDATE device_battery SET device_id = 'fridge-trmnl' WHERE device_id = 'living-room-trmnl';
