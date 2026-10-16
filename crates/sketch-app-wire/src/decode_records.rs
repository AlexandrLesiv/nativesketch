use crate::{reader::Reader, *};
impl Reader<'_> {
    pub fn context(&mut self) -> Result<RawContext, WireError> {
        Ok(RawContext {
            session: self.u64()?,
            document: self.u64()?,
            interaction: self.u64()?,
        })
    }
    pub fn brush(&mut self) -> Result<BrushSettings, WireError> {
        sketch_core::Brush {
            rgba: [self.f64()?, self.f64()?, self.f64()?, self.f64()?],
            width: self.f64()?,
        }
        .try_into()
        .map_err(|_| WireError::malformed())
    }
    pub fn tool(&mut self) -> Result<ToolId, WireError> {
        match self.u32()? {
            1 => Ok(ToolId::Pen),
            2 => Ok(ToolId::Select),
            _ => Err(WireError::malformed()),
        }
    }
    pub fn resource(&mut self) -> Result<ResourceDescriptor, WireError> {
        let provider = ProviderId(self.u32()?);
        let flags = self.u32()?;
        if flags & !7 != 0 {
            return Err(WireError::malformed());
        }
        Ok(ResourceDescriptor {
            provider,
            resource: ProviderResourceId(self.u64()?),
            canonical: ProviderResourceId(self.u64()?),
            generation: self.u64()?,
            access: ResourceAccess {
                read: flags & 1 != 0,
                write: flags & 2 != 0,
                confirmed_write: flags & 4 != 0,
            },
        })
    }
    pub fn viewport(&mut self) -> Result<sketch_core::RenderViewport, WireError> {
        Ok(sketch_core::RenderViewport {
            x: self.f64()?,
            y: self.f64()?,
            width: self.f64()?,
            height: self.f64()?,
            margin: self.f64()?,
        })
    }
    pub fn samples(&mut self) -> Result<Vec<sketch_core::Input>, WireError> {
        let count = usize::try_from(self.u32()?).map_err(|_| WireError::capacity())?;
        self.zero()?;
        if count > 16384 {
            return Err(WireError::capacity());
        }
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(count)
            .map_err(|_| WireError::capacity())?;
        for _ in 0..count {
            let kind = self.u32()?;
            let flags = self.u32()?;
            let id = self.u64()?;
            let x = self.f64()?;
            let y = self.f64()?;
            let timestamp = self.f64()?;
            let pressure = self.f64()?;
            if flags > 1 || (flags == 0 && pressure != 0.0) {
                return Err(WireError::malformed());
            }
            let sample = sketch_core::Sample {
                id,
                x,
                y,
                timestamp,
                pressure: (flags == 1).then_some(pressure),
            };
            samples.push(match kind {
                0 => sketch_core::Input::Actual(sample),
                1 => sketch_core::Input::Predicted(sample),
                2 => sketch_core::Input::Correction(sample),
                _ => return Err(WireError::malformed()),
            });
        }
        Ok(samples)
    }
}
