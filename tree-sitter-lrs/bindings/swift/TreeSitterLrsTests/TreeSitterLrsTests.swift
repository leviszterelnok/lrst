import XCTest
import SwiftTreeSitter
import TreeSitterLrs

final class TreeSitterLrsTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_lrs())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Lrs grammar")
    }
}
