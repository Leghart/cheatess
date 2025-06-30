use opencv::core::Rect;

#[derive(PartialEq, Debug)]
pub struct ScreenRegion {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl ScreenRegion {
    #[allow(dead_code)]
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        ScreenRegion {
            x,
            y,
            width,
            height,
        }
    }

    #[allow(dead_code)]
    pub fn from_rect(rect: Rect) -> Self {
        ScreenRegion {
            x: rect.x,
            y: rect.y,
            width: rect.width as u32,
            height: rect.height as u32,
        }
    }

    #[allow(dead_code)]
    pub fn values(&self) -> (i32, i32, u32, u32) {
        return (self.x, self.y, self.width, self.height);
    }
}

impl TryFrom<String> for ScreenRegion {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim_matches(|c| c == '(' || c == ')');
        let parts: Vec<&str> = trimmed.split(',').collect();

        if parts.len() != 4 {
            return Err(format!(
                "Invalid input: expected 4 elements, got {}",
                parts.len()
            ));
        }

        let x = parts[0].trim().parse::<i32>().map_err(|e| e.to_string())?;
        let y = parts[1].trim().parse::<i32>().map_err(|e| e.to_string())?;
        let width = parts[2].trim().parse::<u32>().map_err(|e| e.to_string())?;
        let height = parts[3].trim().parse::<u32>().map_err(|e| e.to_string())?;

        Ok(ScreenRegion {
            x,
            y,
            width,
            height,
        })
    }
}
