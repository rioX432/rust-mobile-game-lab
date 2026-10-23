import UIKit

final class LobbyViewController: UIViewController {
    private weak var game: GameViewController?
    private var sceneActive = false

    override func viewDidLoad() {
        super.viewDidLoad()
        title = "Rust Mobile Lab"
        view.backgroundColor = .systemBackground
        let button = UIButton(type: .system)
        button.setTitle("Open drawing probe", for: .normal)
        button.addTarget(self, action: #selector(openGame), for: .touchUpInside)
        button.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(button)
        NSLayoutConstraint.activate([
            button.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            button.centerYAnchor.constraint(equalTo: view.centerYAnchor),
        ])
    }

    @objc private func openGame() {
        guard game == nil else { return }
        let controller = GameViewController()
        controller.setSceneActive(sceneActive)
        game = controller
        navigationController?.pushViewController(controller, animated: true)
    }

    func setSceneActive(_ active: Bool) {
        sceneActive = active
        game?.setSceneActive(active)
    }

    func disconnect() { game?.shutdown() }
}

final class GameViewController: UIViewController {
    private var session: RuntimeSession?
    private var sceneActive = false
    private var visible = false
    private let surfaceContainer = UIView()
    private var surface: MetalHostView?
    private let statusView = UITextView()
    private var sizeConstraint: NSLayoutConstraint?

    override func viewDidLoad() {
        super.viewDidLoad()
        title = "Drawing probe"
        view.backgroundColor = .systemBackground
        let session = RuntimeSession()
        self.session = session
        session.onStatus = { [weak self] generation, status in
            guard let self, self.session?.generation == generation else { return }
            self.statusView.text = status
        }

        let heading = UILabel()
        heading.text = "Native UI above Rust: tap the drawing area"
        heading.numberOfLines = 0
        heading.font = .preferredFont(forTextStyle: .headline)
        statusView.isEditable = false
        statusView.font = .monospacedSystemFont(ofSize: 12, weight: .regular)
        statusView.backgroundColor = .secondarySystemBackground
        statusView.accessibilityLabel = "Rust runtime status JSON"

        let reset = button("Reset", action: #selector(resetProbe))
        let resize = button("Resize", action: #selector(resizeSurface))
        let recreate = button("New layer", action: #selector(recreateSurface))
        let close = button("Close", action: #selector(closeGame))
        let controls = UIStackView(arrangedSubviews: [reset, resize, recreate, close])
        controls.distribution = .fillEqually
        controls.spacing = 8
        let stack = UIStackView(arrangedSubviews: [heading, surfaceContainer, controls, statusView])
        stack.axis = .vertical
        stack.spacing = 8
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)
        let sizeConstraint = surfaceContainer.heightAnchor.constraint(equalTo: stack.heightAnchor, multiplier: 0.45)
        self.sizeConstraint = sizeConstraint
        NSLayoutConstraint.activate([
            stack.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 8),
            stack.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -8),
            stack.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 12),
            stack.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -12),
            sizeConstraint,
        ])
        installSurface()
        session.start()
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        visible = true
        session?.setActive(sceneActive)
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        visible = false
        session?.setActive(false)
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)
        if navigationController?.isBeingDismissed == true { shutdown() }
    }

    override func didMove(toParent parent: UIViewController?) {
        super.didMove(toParent: parent)
        // A cancelled interactive pop keeps its parent and must keep its session.
        if parent == nil { shutdown() }
    }

    func setSceneActive(_ active: Bool) {
        sceneActive = active
        session?.setActive(active && visible)
    }

    func shutdown() {
        surface?.onSurfaceChange = nil
        surface?.onTouch = nil
        session?.close()
        session = nil
        surface?.removeFromSuperview()
        surface = nil
    }

    private func button(_ title: String, action: Selector) -> UIButton {
        let button = UIButton(type: .system)
        button.setTitle(title, for: .normal)
        button.addTarget(self, action: action, for: .touchUpInside)
        return button
    }

    private func installSurface() {
        guard let session else { return }
        let surface = MetalHostView(frame: .zero)
        surface.translatesAutoresizingMaskIntoConstraints = false
        surface.onSurfaceChange = { [weak self, weak surface, weak session] layer, metrics in
            guard let self, let surface, self.surface === surface,
                  self.session === session else { return }
            session?.setSurface(layer, metrics: metrics)
        }
        surface.onTouch = { [weak self, weak surface, weak session] x, y in
            guard let self, let surface, self.surface === surface,
                  self.session === session else { return }
            session?.touch(x: x, y: y)
        }
        self.surface = surface
        surfaceContainer.addSubview(surface)
        NSLayoutConstraint.activate([
            surface.topAnchor.constraint(equalTo: surfaceContainer.topAnchor),
            surface.bottomAnchor.constraint(equalTo: surfaceContainer.bottomAnchor),
            surface.leadingAnchor.constraint(equalTo: surfaceContainer.leadingAnchor),
            surface.trailingAnchor.constraint(equalTo: surfaceContainer.trailingAnchor),
        ])
    }

    @objc private func resetProbe() { session?.reset() }

    @objc private func resizeSurface() {
        guard let sizeConstraint, let stack = surfaceContainer.superview else { return }
        let multiplier: CGFloat = sizeConstraint.multiplier > 0.4 ? 0.3 : 0.45
        sizeConstraint.isActive = false
        let replacement = surfaceContainer.heightAnchor.constraint(equalTo: stack.heightAnchor, multiplier: multiplier)
        replacement.isActive = true
        self.sizeConstraint = replacement
        view.setNeedsLayout()
    }

    @objc private func recreateSurface() {
        surface?.onSurfaceChange = nil
        surface?.onTouch = nil
        session?.setSurface(nil, metrics: nil)
        surface?.removeFromSuperview()
        surface = nil
        installSurface()
    }

    @objc private func closeGame() {
        shutdown()
        navigationController?.popViewController(animated: true)
    }
}
