use super::i2c;
use num::FromPrimitive;
use num_derive::FromPrimitive;

pub enum Model {
    Conexant,
    Focus,
    Xcalibur,
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum AVMode {
    RGBScart = 0x0,
    HDTV = 0x1,
    VGASyncOnGreen = 0x2,
    SVideo = 0x4,
    Composite = 0x6,
    VGA = 0x7,
}

pub struct VideoModeInfo {
    pub width: u32,
    pub height: u32,
    pub xmargin: u32,
    pub ymargin: u32,
    pub nvhtotal: u32,
    pub nvvtotal: u32,
    pub nvhstart: u32,
    pub nvvstart: u32,
    pub pixel_depth: u32,

    pub crtc_hend: u32,
    pub crtc_vstart: u32,
    pub crtc_vtotal: u32,
}

pub trait Encoder {}

impl Model {
    pub fn is_xcalibur(&self) -> bool {
        match self {
            Model::Xcalibur => true,
            _ => false,
        }
    }

    pub fn detect() -> Self {
        // Safety: each encoder addr/reg should exist
        // or error appropriately
        unsafe {
            if i2c::tx_read(0x45, 0x0).is_ok() {
                return Self::Conexant;
            }

            if i2c::tx_read(0x6a, 0x0).is_ok() {
                return Self::Focus;
            }
        }

        Self::Xcalibur
    }
}

impl AVMode {
    pub fn detect() -> Self {
        let mode = unsafe { i2c::tx_read(0x10, 0x4) };
        mode.ok()
            .map(|mode| AVMode::from_u32(mode))
            .flatten()
            .unwrap_or(AVMode::Composite)
    }

    fn get_vm_hdtv(&self, enc: &Model) -> Option<VideoModeInfo> {
        // FIXME: Support more than 480p

        let (nvhtotal, nvvtotal) = if enc.is_xcalibur() {
            (779, 524)
        } else {
            (858, 525)
        };

        Some(VideoModeInfo {
            width: 720,
            height: 480,
            xmargin: 0,
            ymargin: 0,
            nvhtotal,
            nvvtotal,
            nvhstart: 738,
            nvvstart: 489,
            pixel_depth: (32 + 1) / 8,

            crtc_hend: 720,
            crtc_vstart: 489,
            crtc_vtotal: nvvtotal,
        })
    }

    pub fn get_video_mode(&self, enc: &Model) -> Option<VideoModeInfo> {
        match self {
            Self::HDTV => self.get_vm_hdtv(enc),
            _ => None,
        }
    }
}
