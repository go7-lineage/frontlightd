# frontlightd

Frontlight brightness and warmth daemon for the Boox Go 7 on LineageOS

## Using in a device tree

1. Check it out next to the device, such as in a local manifest:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<manifest>
    <project name="go7-lineage/frontlightd" path="hardware/onyx/frontlight" remote="github" revision="master" />
    ...
</manifest>
```

2. In `device.mk`, `PRODUCT_PACKAGES += frontlightd`
3. In `BoardConfig.mk`, `BOARD_VENDOR_SEPOLICY_DIRS += hardware/onyx/frontlight/sepolicy`
