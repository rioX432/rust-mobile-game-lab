import UIKit

@main
final class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        true
    }
}

final class SceneDelegate: UIResponder, UIWindowSceneDelegate {
    var window: UIWindow?
    private var lobby: LobbyViewController?

    func scene(
        _ scene: UIScene,
        willConnectTo session: UISceneSession,
        options connectionOptions: UIScene.ConnectionOptions
    ) {
        guard let scene = scene as? UIWindowScene else { return }
        let lobby = LobbyViewController()
        let window = UIWindow(windowScene: scene)
        window.rootViewController = UINavigationController(rootViewController: lobby)
        self.lobby = lobby
        self.window = window
        window.makeKeyAndVisible()
    }

    func sceneDidBecomeActive(_ scene: UIScene) { lobby?.setSceneActive(true) }
    func sceneWillResignActive(_ scene: UIScene) { lobby?.setSceneActive(false) }
    func sceneDidEnterBackground(_ scene: UIScene) { lobby?.setSceneActive(false) }
    func sceneDidDisconnect(_ scene: UIScene) { lobby?.disconnect() }
}
