import {
  Box,
  Container,
  Toolbar,
  createTheme,
  useMediaQuery,
} from "@mui/material";
import { useCallback, useMemo, useState } from "react";
import { useUpdateAvailableToast } from "../hooks/useUpdateAvailableToast";
import { ConnectedAppBar } from "./ConnectedAppBar";
import { DisconnectedAppBar } from "./DisconnectedAppBar";
import { HomePage } from "./HomePage";
import { DeviceSettings } from "./deviceSettings/DeviceSettings";
import { Device, selectDemoDevice, selectDevice } from "../bluetooth/Device";
import { LoadingScreen } from "./LoadingScreen";

export function AppContents() {
  const [device, setDevice] = useState<Device>();
  const [isLoading, setLoading] = useState(false);
  const prefersDarkMode = useMediaQuery("(prefers-color-scheme: dark)");
  const isDemoMode = localStorage.getItem("openscq30:demoMode") == "true";
  const theme = useMemo(
    () =>
      createTheme({
        palette: {
          mode: prefersDarkMode ? "dark" : "light",
        },
      }),
    [prefersDarkMode],
  );
  useUpdateAvailableToast();

  const connect = useCallback(() => {
    setLoading(true);
    (isDemoMode ? selectDemoDevice : selectDevice)()
      .then(setDevice)
      .catch((err) => {
        setLoading(false);
        // Ignore error if the user canceled the device selection popup
        if (!(err instanceof DOMException) || err.name != "NotFoundError") {
          console.error(err);
        }
      });
  }, [isDemoMode]);

  const disconnect = useCallback(() => {
    device?.destroy();
    setDevice(undefined);
    setLoading(false);
  }, [device]);

  return (
    <>
      <Box
        sx={{
          display: "flex",
          backgroundColor: theme.palette.background.default,
          minHeight: "100vh",
        }}
        color="text.primary"
      >
        {device ? (
          <ConnectedAppBar
            deviceName={device.name ?? "Unknown device"}
            onDisconnectClick={() => disconnect()}
          />
        ) : (
          <DisconnectedAppBar
            onSelectDeviceClick={() => connect()}
            showSelectDeviceButton={!!navigator.bluetooth || isDemoMode}
          />
        )}
        <Box component="main" sx={{ flexGrow: 1 }}>
          <Toolbar />
          <Container sx={{ my: 2 }}>
            {device ? (
              <DeviceSettings device={device} disconnect={disconnect} />
            ) : isLoading ? (
              <LoadingScreen />
            ) : (
              <HomePage />
            )}
          </Container>
        </Box>
      </Box>
    </>
  );
}

export default AppContents;
