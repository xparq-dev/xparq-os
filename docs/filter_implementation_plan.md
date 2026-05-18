# Nebula Story Camera: Filter Implementation Plan

This document outlines the strategic roadmap for the Nebula Story Camera's filter system, ranging from basic color grading to advanced AI-driven face transformations.

---

## Phase 1: Cinematic Color Grading (Current State)
**Status:** ✅ Implemented
**Objective:** High-performance aesthetic color transformation using 3D LUTs.

### Technical Architecture
- **GPU Engine:** Fragment Shaders (GLSL) via Flutter's `ui.FragmentProgram`.
- **Data Source:** Adobe Standard `.cube` files (33x33x33 resolution).
- **Efficiency:** Processing happens at the pixel level on the GPU, ensuring 60fps performance even on mid-range devices.

### Next Steps for Phase 1
- **Dynamic Registry:** Move filter metadata to a JSON config for easy OTA (Over-the-Air) updates.
- **Intensity Slider:** Implement a `uIntensity` uniform in the shader to allow users to adjust filter strength (0% to 100%).

---

## Phase 2: AI Face Beauty & Smart Retouching (Short-term)
**Status:** 💡 Proposed
**Objective:** Enhance portraits automatically while maintaining a natural look.

### Core Features
1. **Skin Smoothing:** Removing blemishes and evening skin tone without losing texture.
2. **Face Reshaping:** Subtle adjustments to chin, eyes, and nose.
3. **Brightness Calibration:** Detecting the face area and adjusting local exposure (Digital Fill Light).

### Technical Implementation
- **Detection:** Use `google_mlkit_face_detection` to get real-time face contours.
- **Hybrid Shader:**
    - **Step 1:** Generate a "Skin Mask" based on the face landmarks.
    - **Step 2:** Apply a **Bilateral Filter** or **Surface Blur** only within the skin mask region.
    - **Step 3:** Composite the smoothed skin back onto the original image to preserve eyes and lips sharpness.

---

## Phase 3: Augmented Reality (AR) Face Masks (Mid-term)
**Status:** 🚀 Planned
**Objective:** Interactive 2D/3D overlays that track user movement.

### Core Features
- **2D Stickers:** Glasses, hats, or dynamic "Portrait Frames" that follow the head.
- **3D Masks:** Full-face 3D models (e.g., Space Helmet, Nebula Aura).
- **Face Mesh:** 468-point tracking for micro-expressions (mouth opening, blinking).

### Technical Options
1. **DIY Approach (Flutter Native):**
    - Use `google_mlkit_face_mesh` for 468-point tracking.
    - Use `flutter_scene` (3D engine) to anchor objects to mesh indices.
2. **SDK Approach (Recommended for Speed/Quality):**
    - **DeepAR SDK:** Allows importing Spark AR-like effects. High cost but industry-standard quality.
    - **Banuba SDK:** Excellent for makeup and hair coloring.

---

## Phase 4: Generative AI Face Transformation (Long-term)
**Status:** 🔬 Research
**Objective:** Using Neural Networks to create new facial appearances.

### Use Cases
- **Style Transfer:** Changing the image to look like a painting or a 3D cartoon (Disney style).
- **Aging/Gender Swap:** Predictive facial restructuring using GANs (Generative Adversarial Networks).
- **Background Replacement:** Intelligent segmentation to put the user in "Deep Space" without a green screen.

### Technical Challenges
- **Compute Power:** On-device inference requires highly optimized **TFLite (TensorFlow Lite)** models.
- **Latency:** Real-time AI transformation usually requires a NPU (Neural Processing Unit) found in high-end chips (Apple A-series, Snapdragon 8 Gen 2+).
- **Fallback:** For older devices, processing must be handled via Cloud Inference (XPARQ Backend).

---

## 🛠️ Recommended Stack for XPARQ
| Feature | Recommended Tech | Reason |
| :--- | :--- | :--- |
| **Face Detection** | Google ML Kit | Free, Fast, works offline, best for Flutter. |
| **Beauty Shaders** | GLSL (GL Subsurface) | Low overhead, preserves GPU battery. |
| **AI Models** | TensorFlow Lite | Industry standard for mobile AI. |
| **3D Rendering** | ThreeDart / Scene | Good balance between power and Flutter integration. |

---

## 🛡️ Privacy & Performance Notes
- **Local Processing:** All Phase 1, 2, and 3 features should ideally run **on-device** to ensure maximum privacy for XPARQ users.
- **Battery Optimization:** Shaders should be cached, and face detection frequency should be throttled when the camera is static to save power.
- **Data Security:** Sensitive biometric data from Face Mesh should NEVER be stored or transmitted; only the final rendered image/video is kept.
