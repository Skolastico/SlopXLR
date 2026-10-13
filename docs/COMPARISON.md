# SlopXLR compared with GoXLR Utility

[Documentation index](../README.md#documentation)

SlopXLR is a derivative of [GoXLR Utility](https://github.com/GoXLR-on-Linux/goxlr-utility), initially based on 49ab4e4. This comparison describes the inherited source/interface included at that base, rather than every future upstream version. The native window source comes from [goxlr-utility-ui](https://github.com/FrostyCoolSlug/goxlr-utility-ui), base e71bb2e.

The service, USB protocol, profile formats and standard hardware audio controls remain rooted in upstream. SlopXLR adds functionality and reorganizes the frontend around that service.

## Feature comparison

| Area | Inherited Utility baseline | SlopXLR change |
| --- | --- | --- |
| Mixing | Device channels, output volumes, mute and supported submix. | Reorganized Mixer and dedicated hardware-fader configuration. |
| Microphone | Type/gain, model-specific EQ, gate, compressor and processing. | Signal/history view, explicit simple/detailed modes and frontend-local EQ A/B snapshots. |
| Routing | Device input/output routing and monitoring. | Matrix view and software-soundboard explanation for the Mini's System route. |
| Normal lighting | Colors, styles and supported model-specific groups/displays. | Reorganized editing, group application and temporary feedback for custom modes. |
| Device/mic profiles | Load/save/create profiles and library folders. | Dedicated Profiles and files area and explicit save guidance. |
| Mini button behavior | Standard mute, Bleep and Cough functions. | Optional action pages with five bindings per page and long/double/triple Cough navigation. |
| Windows application audio | Standard device channel mixing. | Per-executable Windows audio-session control, four Mini positions per page, channel associations, mute, reference volumes and pickup. |
| Mini sampler | No physical full-size sampler controls. | Three software banks with four slots each, Play/Stop actions, multiple clips, order and non-destructive trimming. |
| Full-size effects/sampler | Existing physical voice effects and hardware sampler. | Exposed where supported in the new interface; no claim of new Mini hardware capability. |
| Startup fader state | Inherited Mini initialization interpreted the initial USB reading as movement in the reproduced case. | Initial reading establishes a reference without overwriting restored volumes; later movement controls volume. |
| Device profile failures | Existing USB/profile loading behavior. | Recovery paths and tests around partial application. |
| Frontend state/errors | Original bundled frontend and its transport. | Real service notifications, draft handling, model capability checks and clearer recovery states. |
| Languages | Inherited frontend/service messages. | English and Portuguese authored UI catalogs with user-entered names/paths preserved. |
| Previous interface | Original embedded frontend. | Retained under Settings → Advanced options for access during migration. |
| Distribution | Upstream installers/packages and native wrapper. | SlopXLR per-user setup, portable helper, copied imports, retained data on uninstall and signed update manifests. |

## Model/platform differences

| Capability | GoXLR Mini | Full-size GoXLR |
| --- | --- | --- |
| Standard mixer, microphone and routing | Available; connected-device checks performed. | Retained in source; physical validation pending. |
| Custom action pages | Supported by the service. | Not currently supported. |
| Windows Applications mode | Supported by the service. | Not currently supported. |
| Software soundboard | Virtual banks through System. | Existing full-size sampler path is used. |
| Voice effects, scribble displays and physical recording | Not provided by Mini hardware. | Controls retained; physical validation pending. |

The underlying upstream project supports other operating systems. This fork's custom Windows audio control, installer and release validation target Windows x64. A source-level inherited platform path is not a validated SlopXLR release.

## Migration and behavioral differences

- Imports copy the original configuration and rewrite only the copied library locations. The original Utility installation remains intact.
- The two services cannot run concurrently on the same local port/device. Close one before opening the other.
- Custom action navigation reserves the long Cough gesture while enabled. Disable it for original long-press behavior.
- Applications mode changes the role of all four Mini faders temporarily. Lower them for preparation and follow pickup indicators before expecting them to change an application's volume.
- Audio/device changes use profile saving; custom action and application lists use their own Save controls.
- Closing the native window is different from stopping the service.
- Windows executable and data-folder identifiers retain some upstream/first-beta names for compatibility. The displayed product is SlopXLR.

## Evidence and limits

The frontend migration was checked against the original frontend command usage and the shared IPC contracts. Automated tests cover representable effect values, model gating, routing direction, serialization, action and profile recovery paths. They do not prove every control's acoustic behavior or full-size hardware compatibility.

See [validation](VALIDATION.md) for the current checks, [the user guide](USER_GUIDE.md) for operation and [historical Portuguese notes](history/pt-PT/COMPARACAO-INTERFACES.md) for the development audit. Historical notes describe the state at their own date; the guides above describe the current intended behavior.
