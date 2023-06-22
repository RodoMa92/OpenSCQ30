import { SoundcoreDeviceUtils } from "../../wasm/pkg/openscq30_web_wasm";
import { Observable } from "rxjs";

export class SoundcoreDeviceConnection {
  public readonly incomingPackets: Observable<Uint8Array>;

  private gatt: BluetoothRemoteGATTServer;
  private writeCharacteristic: BluetoothRemoteGATTCharacteristic;
  private readCharacteristic: BluetoothRemoteGATTCharacteristic;

  constructor(
    gatt: BluetoothRemoteGATTServer,
    writeCharacteristic: BluetoothRemoteGATTCharacteristic,
    readCharacteristic: BluetoothRemoteGATTCharacteristic,
  ) {
    this.gatt = gatt;
    this.writeCharacteristic = writeCharacteristic;
    this.readCharacteristic = readCharacteristic;

    this.incomingPackets = new Observable((subscriber) => {
      const handler = () => {
        if (readCharacteristic.value) {
          subscriber.next(new Uint8Array(readCharacteristic.value.buffer));
        } else {
          console.error(
            "Read characteristic value changed, but it is undefined?",
          );
        }
      };
      this.readCharacteristic.addEventListener(
        "characteristicvaluechanged",
        handler,
      );
      return () =>
        readCharacteristic.removeEventListener(
          "characteristicvaluechanged",
          handler,
        );
    });
  }

  public get connected() {
    return this.gatt.connected;
  }

  public async reconnect() {
    if (!this.connected) {
      this.gatt = await this.gatt.connect();
      const service = await this.gatt.getPrimaryService(
        SoundcoreDeviceUtils.getServiceUuid(),
      );
      [this.writeCharacteristic, this.readCharacteristic] = await Promise.all([
        service.getCharacteristic(
          SoundcoreDeviceUtils.getWriteCharacteristicUuid(),
        ),
        service.getCharacteristic(
          SoundcoreDeviceUtils.getReadCharacteristicUuid(),
        ),
      ]);
      await this.readCharacteristic.startNotifications();
    }
  }

  public disconnect() {
    this.readCharacteristic.removeEventListener(
      "characteristicvaluechanged",
      null,
    );
    this.gatt.disconnect();
  }

  public get name() {
    return this.gatt.device.name;
  }

  public async write(value: BufferSource) {
    if (import.meta.env.DEV) {
      console.log("Writing packet at " + new Date().toISOString());
    }
    await this.writeCharacteristic.writeValueWithoutResponse(value);
  }
}

export async function selectDeviceConnection(): Promise<SoundcoreDeviceConnection> {
  const serviceUuid = SoundcoreDeviceUtils.getServiceUuid();
  const macAddressPrefixes = SoundcoreDeviceUtils.getMacAddressPrefixes();
  const device = await navigator.bluetooth.requestDevice({
    // We would filter by available services, but this doesn't seem to work on chromium based browsers on platforms
    // other than Linux without first going to about://bluetooth-internals/#devices, scanning for your device, and
    // then inspecting it.
    // filters: [{ services: [serviceUuid] }],
    filters: macAddressPrefixes.map((prefix) => ({
      manufacturerData: [
        {
          // Non standard manufacturer data format: mac address followed by 0x00 0x00
          // companyIdentifier is picked up as the first two bytes of the mac address
          companyIdentifier: (prefix[1] << 8) | prefix[0],
          // data is everything after those first two bytes. Since we want to filter by the first three bytes of the
          // mac address, that just leaves one more byte.
          dataPrefix: Uint8Array.of(prefix[2]),
        },
      ],
    })),
    optionalServices: [serviceUuid],
  });
  if (device.gatt == undefined) {
    throw new Error("Bluetooth device does not support GATT");
  }
  const gatt = await device.gatt.connect();
  const service = await gatt.getPrimaryService(
    SoundcoreDeviceUtils.getServiceUuid(),
  );
  const [writeCharacteristic, readCharacteristic] = await Promise.all([
    service.getCharacteristic(
      SoundcoreDeviceUtils.getWriteCharacteristicUuid(),
    ),
    service.getCharacteristic(SoundcoreDeviceUtils.getReadCharacteristicUuid()),
  ]);
  await readCharacteristic.startNotifications();

  return new SoundcoreDeviceConnection(
    gatt,
    writeCharacteristic,
    readCharacteristic,
  );
}
