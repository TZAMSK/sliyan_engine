use crate::scene::camera::Camera;

pub trait ViewStrategy {
    fn apply(camera: &mut Camera);
}

pub struct FrontView;
pub struct IsometricView;
pub struct OrthographicView;
pub struct SideView;
pub struct TopView;

impl ViewStrategy for FrontView {
    fn apply(camera: &mut Camera) {
        camera.set_view(
            glm::vec3(0.0, -12.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
        );
    }
}

impl ViewStrategy for IsometricView {
    fn apply(camera: &mut Camera) {
        camera.set_view(
            glm::vec3(5.0, -8.0, 4.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
        );
    }
}

impl ViewStrategy for OrthographicView {
    fn apply(camera: &mut Camera) {
        camera.set_view(
            glm::vec3(50.0, -80.0, 40.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
        );
    }
}

impl ViewStrategy for SideView {
    fn apply(camera: &mut Camera) {
        camera.set_view(
            glm::vec3(12.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
        );
    }
}

impl ViewStrategy for TopView {
    fn apply(camera: &mut Camera) {
        camera.set_view(
            glm::vec3(0.0, 0.0, 20.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
        );
    }
}

pub struct CameraViewSelector;

impl CameraViewSelector {
    pub fn select<S: ViewStrategy>(camera: &mut Camera) {
        S::apply(camera);
    }
}
