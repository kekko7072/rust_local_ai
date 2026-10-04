import Foundation

#if canImport(FoundationModels)
import FoundationModels
#endif

private func copyCString(_ value: String) -> UnsafeMutablePointer<CChar>? {
  strdup(value)
}

@_cdecl("rla_string_free")
public func rlaStringFree(_ value: UnsafeMutablePointer<CChar>?) {
  free(value)
}

/// 0 available, 1 OS too old/build unavailable, 2 hardware, 3 disabled,
/// 4 model not ready, 5 other.
@_cdecl("rla_apple_availability")
public func rlaAppleAvailability(
  _ detail: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
  #if canImport(FoundationModels)
  guard #available(macOS 26.0, *) else {
    detail?.pointee = copyCString("Apple Foundation Models requires macOS 26 or newer.")
    return 1
  }
  switch SystemLanguageModel.default.availability {
  case .available:
    detail?.pointee = copyCString("Apple Foundation Models is ready.")
    return 0
  case .unavailable(.deviceNotEligible):
    detail?.pointee = copyCString("This Mac is not eligible for Apple Intelligence.")
    return 2
  case .unavailable(.appleIntelligenceNotEnabled):
    detail?.pointee = copyCString("Apple Intelligence is disabled in System Settings.")
    return 3
  case .unavailable(.modelNotReady):
    detail?.pointee = copyCString("The system model is downloading or preparing.")
    return 4
  case .unavailable(let reason):
    detail?.pointee = copyCString("Apple Foundation Models is unavailable: \(String(reflecting: reason))")
    return 5
  }
  #else
  detail?.pointee = copyCString("This build SDK does not contain Foundation Models.")
  return 1
  #endif
}

/// Capability bits implemented by this bridge, not merely by the OS API.
@_cdecl("rla_apple_capabilities")
public func rlaAppleCapabilities() -> UInt32 {
  #if canImport(FoundationModels)
  guard #available(macOS 26.0, *) else { return 0 }
  var result: UInt32 = (1 << 0) | (1 << 5) | (1 << 6) | (1 << 7)
  if #available(macOS 26.4, *) { result |= (1 << 4) }
  return result
  #else
  return 0
  #endif
}

#if canImport(FoundationModels)
@available(macOS 26.0, *)
private final class AppleSession: @unchecked Sendable {
  let session: LanguageModelSession
  private let lock = NSLock()
  private var prompt = ""
  private var task: Task<Void, Never>?

  init(instructions: String?) {
    session = LanguageModelSession(instructions: instructions)
  }

  func append(_ value: String) {
    lock.lock()
    prompt += value
    lock.unlock()
  }

  func takePrompt() -> String {
    lock.lock()
    defer { lock.unlock() }
    let value = prompt
    prompt = ""
    return value
  }

  func setTask(_ value: Task<Void, Never>?) {
    lock.lock()
    task = value
    lock.unlock()
  }

  func cancel() {
    lock.lock()
    let current = task
    lock.unlock()
    current?.cancel()
  }
}
#endif

@_cdecl("rla_apple_session_create")
public func rlaAppleSessionCreate(
  _ instructions: UnsafePointer<CChar>?,
  _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
  #if canImport(FoundationModels)
  guard #available(macOS 26.0, *) else {
    errorOut?.pointee = copyCString("Apple Foundation Models requires macOS 26 or newer.")
    return nil
  }
  let text = instructions.map { String(cString: $0) }
  return Unmanaged.passRetained(AppleSession(instructions: text)).toOpaque()
  #else
  errorOut?.pointee = copyCString("This build SDK does not contain Foundation Models.")
  return nil
  #endif
}

@_cdecl("rla_apple_session_add")
public func rlaAppleSessionAdd(
  _ opaque: UnsafeMutableRawPointer?, _ text: UnsafePointer<CChar>?
) -> Int32 {
  #if canImport(FoundationModels)
  guard #available(macOS 26.0, *), let opaque, let text else { return 1 }
  let session = Unmanaged<AppleSession>.fromOpaque(opaque).takeUnretainedValue()
  session.append(String(cString: text))
  return 0
  #else
  return 1
  #endif
}

@_cdecl("rla_apple_session_generate")
public func rlaAppleSessionGenerate(
  _ opaque: UnsafeMutableRawPointer?,
  _ temperature: Double,
  _ topP: Double,
  _ maximumTokens: Int32,
  _ output: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
  _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
  #if canImport(FoundationModels)
  guard #available(macOS 26.0, *), let opaque else {
    errorOut?.pointee = copyCString("Apple Foundation Models is unavailable.")
    return 1
  }
  let holder = Unmanaged<AppleSession>.fromOpaque(opaque).takeUnretainedValue()
  let prompt = holder.takePrompt()
  let semaphore = DispatchSemaphore(value: 0)
  let task = Task {
    defer { semaphore.signal() }
    do {
      let max = maximumTokens > 0 ? Int(maximumTokens) : nil
      let options: GenerationOptions
      if topP >= 0 {
        options = GenerationOptions(
          sampling: .random(probabilityThreshold: topP),
          temperature: temperature >= 0 ? temperature : nil,
          maximumResponseTokens: max)
      } else if temperature >= 0 {
        options = GenerationOptions(temperature: temperature, maximumResponseTokens: max)
      } else {
        options = GenerationOptions(sampling: .greedy, maximumResponseTokens: max)
      }
      let response = try await holder.session.respond(to: prompt, options: options)
      try Task.checkCancellation()
      output?.pointee = copyCString(response.content)
    } catch is CancellationError {
      errorOut?.pointee = copyCString("CANCELLED")
    } catch {
      errorOut?.pointee = copyCString(error.localizedDescription)
    }
  }
  holder.setTask(task)
  semaphore.wait()
  holder.setTask(nil)
  return output?.pointee == nil ? 1 : 0
  #else
  errorOut?.pointee = copyCString("This build SDK does not contain Foundation Models.")
  return 1
  #endif
}

@_cdecl("rla_apple_session_cancel")
public func rlaAppleSessionCancel(_ opaque: UnsafeMutableRawPointer?) {
  #if canImport(FoundationModels)
  guard #available(macOS 26.0, *), let opaque else { return }
  Unmanaged<AppleSession>.fromOpaque(opaque).takeUnretainedValue().cancel()
  #endif
}

@_cdecl("rla_apple_session_destroy")
public func rlaAppleSessionDestroy(_ opaque: UnsafeMutableRawPointer?) {
  #if canImport(FoundationModels)
  guard #available(macOS 26.0, *), let opaque else { return }
  let session = Unmanaged<AppleSession>.fromOpaque(opaque).takeRetainedValue()
  session.cancel()
  #endif
}

@_cdecl("rla_apple_count_tokens")
public func rlaAppleCountTokens(
  _ text: UnsafePointer<CChar>?,
  _ output: UnsafeMutablePointer<UInt64>?,
  _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
  #if canImport(FoundationModels) && compiler(>=6.3)
  guard #available(macOS 26.4, *), let text else {
    errorOut?.pointee = copyCString("Exact token counting requires macOS 26.4 or newer.")
    return 1
  }
  let value = String(cString: text)
  let semaphore = DispatchSemaphore(value: 0)
  Task {
    defer { semaphore.signal() }
    do { output?.pointee = UInt64(try await SystemLanguageModel.default.tokenCount(for: value)) }
    catch { errorOut?.pointee = copyCString(error.localizedDescription) }
  }
  semaphore.wait()
  return errorOut?.pointee == nil ? 0 : 1
  #else
  errorOut?.pointee = copyCString("Exact token counting is unavailable in this build SDK.")
  return 1
  #endif
}
