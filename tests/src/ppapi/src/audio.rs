
use super::interface::*;
use super::sys::*;

static AUDIO_INTERFACE: PPB_Audio_1_1 = PPB_Audio_1_1 {
    create: ret_default_stub::<_>,
    is_audio: ret_false_stub,
    get_config: ret_default_stub::<_>,
    start_playback: ret_false_stub,
    stop_playback: ret_false_stub,
};

static AUDIO_CONFIG_INTERFACE: PPB_AudioConfig_1_1 = PPB_AudioConfig_1_1 {
    CreateStereo16Bit: ret_default_stub::<_>,
    RecommendSampleFrameCount: ret_default_stub::<_>,
    IsAudioConfig: ret_false_stub,
    GetSampleRate: ret_default_stub::<_>,
    GetSampleFrameCount: ret_default_stub::<_>,
    RecommendSampleRate: ret_default_stub::<_>,
};

pub static INTERFACES: Interfaces = &[
    ("PPB_Audio;1.1", interface_ptr(&AUDIO_INTERFACE)),
    ("PPB_AudioConfig;1.1", interface_ptr(&AUDIO_CONFIG_INTERFACE)),
];
