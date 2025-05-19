// james was here
const { invoke, Channel } = window.__TAURI__.core;

var connecting = false;
var connected = false;

function isMuse(device) {
  return device.name && device.name.toLowerCase().includes("muse");
}

const onDevices = new Channel();
const onDisconnect = new Channel();

onDisconnect.onmessage = (message) => {};

onDevices.onmessage = (message) => {
  if (!connected && !connecting) {
    console.log(message);
    document.getElementById("bluetoothStatus").style.display = "block";
    document.getElementById("bluetoothStatus").innerHTML =
      "Searching for Muse... (" + message.length + " BLE devices so far)";

    for (const device of message) {
      if (isMuse(device)) {
        console.log("Found Muse device:", device);
        connecting = true;
        document.getElementById("bluetooth").innerHTML = "Connecting...";
        document.getElementById("bluetoothStatus").innerHTML =
          "Muse found! Connecting to " + device.name + "...";
        document.getElementById("bluetooth").style.backgroundColor =
          "rgb(33, 33, 108)";
        invoke("plugin:blec|connect", { address: device.address, onDisconnect })
          .then(() => {
            connected = true;
            connecting = false;
            document.getElementById("bluetooth").innerHTML = "Connected";
            document.getElementById("bluetoothStatus").innerHTML =
              "Connected to " + device.name;
            document.getElementById("bluetooth").style.backgroundColor =
              "rgb(33, 109, 30)";
            document.getElementById("connectWait").style.display = "none";
            document.getElementById("streamButtons").style.display = "block";
          })
          .catch((e) => {
            document.getElementById("bluetooth").innerHTML = "Connection Error";
            document.getElementById("bluetooth").style.backgroundColor =
              "rgb(118, 46, 42)";
            document.getElementById("bluetoothStatus").innerHTML =
              "Muse connection failed: " + e;
          });
      }
    }
  }
};

document.getElementById("streamOSC").addEventListener("click", function () {
  var ip = document.getElementById("ipAddr").value;
  var port = document.getElementById("port").value;

  if (connected) {
    document.getElementById("ipAddr").disabled = true;
    document.getElementById("port").disabled = true;

    invoke("stream_osc", { ip: ip, port_str: port });
    document.getElementById("streamOSC").disabled = true;
    document.getElementById("streamOSC").innerHTML = "Streaming OSC";
    document.getElementById("streamOSC").style.backgroundColor =
      "rgb(33, 109, 30)";
  }
});

document.getElementById("streamLSL").addEventListener("click", function () {
  var name = document.getElementById("saveName").value;

  if (connected) {
    document.getElementById("saveName").disabled = true;

    invoke("stream_lsl", { stream_name: name });
    document.getElementById("streamLSL").disabled = true;
    document.getElementById("streamLSL").innerHTML = "Streaming LSL";
    document.getElementById("streamLSL").style.backgroundColor =
      "rgb(33, 109, 30)";
  }
});

document.getElementById("bluetooth").addEventListener("click", function () {
  document.getElementById("bluetooth").innerHTML = "Discovering...";
  document.getElementById("bluetoothStatus").innerHTML = "Scanning for Muse...";
  document.getElementById("bluetooth").style.backgroundColor =
    "rgb(83, 50, 75)";
  invoke("plugin:blec|scan", {
    timeout: 5000,
    onDevices,
  });
  setTimeout(function () {
    if (!connected && !connecting) {
      document.getElementById("bluetooth").innerHTML = "Try Again";
      document.getElementById("bluetooth").style.backgroundColor =
        "rgb(118, 46, 42)";
      document.getElementById("bluetoothStatus").innerHTML = "Muse not found.";
    }
  }, 5500);
});
