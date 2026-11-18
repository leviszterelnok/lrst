package tree_sitter_lrs_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_lrs "github.com/tree-sitter/tree-sitter-lrs/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_lrs.Language())
	if language == nil {
		t.Errorf("Error loading Lrs grammar")
	}
}
