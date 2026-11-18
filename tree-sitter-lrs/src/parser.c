#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 45
#define LARGE_STATE_COUNT 14
#define SYMBOL_COUNT 28
#define ALIAS_COUNT 0
#define TOKEN_COUNT 15
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 3
#define MAX_ALIAS_SEQUENCE_LENGTH 4
#define PRODUCTION_ID_COUNT 4

enum ts_symbol_identifiers {
  sym_identifier = 1,
  anon_sym_include = 2,
  anon_sym_SEMI = 3,
  anon_sym_namespace = 4,
  anon_sym_DOT = 5,
  anon_sym_LBRACE = 6,
  anon_sym_RBRACE = 7,
  sym_punct = 8,
  sym_number = 9,
  sym_string = 10,
  sym_char = 11,
  sym_lifetime = 12,
  sym_line_comment = 13,
  sym_block_comment = 14,
  sym_source_file = 15,
  sym__top = 16,
  sym_include = 17,
  sym_namespace = 18,
  sym_namespace_path = 19,
  sym_namespace_body = 20,
  sym__token = 21,
  sym_block = 22,
  sym_dot = 23,
  aux_sym_source_file_repeat1 = 24,
  aux_sym_namespace_path_repeat1 = 25,
  aux_sym_namespace_body_repeat1 = 26,
  aux_sym_block_repeat1 = 27,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_identifier] = "identifier",
  [anon_sym_include] = "include",
  [anon_sym_SEMI] = ";",
  [anon_sym_namespace] = "namespace",
  [anon_sym_DOT] = ".",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [sym_punct] = "punct",
  [sym_number] = "number",
  [sym_string] = "string",
  [sym_char] = "char",
  [sym_lifetime] = "lifetime",
  [sym_line_comment] = "line_comment",
  [sym_block_comment] = "block_comment",
  [sym_source_file] = "source_file",
  [sym__top] = "_top",
  [sym_include] = "include",
  [sym_namespace] = "namespace",
  [sym_namespace_path] = "namespace_path",
  [sym_namespace_body] = "namespace_body",
  [sym__token] = "_token",
  [sym_block] = "block",
  [sym_dot] = "dot",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_namespace_path_repeat1] = "namespace_path_repeat1",
  [aux_sym_namespace_body_repeat1] = "namespace_body_repeat1",
  [aux_sym_block_repeat1] = "block_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_identifier] = sym_identifier,
  [anon_sym_include] = anon_sym_include,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_namespace] = anon_sym_namespace,
  [anon_sym_DOT] = anon_sym_DOT,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [sym_punct] = sym_punct,
  [sym_number] = sym_number,
  [sym_string] = sym_string,
  [sym_char] = sym_char,
  [sym_lifetime] = sym_lifetime,
  [sym_line_comment] = sym_line_comment,
  [sym_block_comment] = sym_block_comment,
  [sym_source_file] = sym_source_file,
  [sym__top] = sym__top,
  [sym_include] = sym_include,
  [sym_namespace] = sym_namespace,
  [sym_namespace_path] = sym_namespace_path,
  [sym_namespace_body] = sym_namespace_body,
  [sym__token] = sym__token,
  [sym_block] = sym_block,
  [sym_dot] = sym_dot,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_namespace_path_repeat1] = aux_sym_namespace_path_repeat1,
  [aux_sym_namespace_body_repeat1] = aux_sym_namespace_body_repeat1,
  [aux_sym_block_repeat1] = aux_sym_block_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_include] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SEMI] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_namespace] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [sym_punct] = {
    .visible = true,
    .named = true,
  },
  [sym_number] = {
    .visible = true,
    .named = true,
  },
  [sym_string] = {
    .visible = true,
    .named = true,
  },
  [sym_char] = {
    .visible = true,
    .named = true,
  },
  [sym_lifetime] = {
    .visible = true,
    .named = true,
  },
  [sym_line_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_block_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym__top] = {
    .visible = false,
    .named = true,
  },
  [sym_include] = {
    .visible = true,
    .named = true,
  },
  [sym_namespace] = {
    .visible = true,
    .named = true,
  },
  [sym_namespace_path] = {
    .visible = true,
    .named = true,
  },
  [sym_namespace_body] = {
    .visible = true,
    .named = true,
  },
  [sym__token] = {
    .visible = false,
    .named = true,
  },
  [sym_block] = {
    .visible = true,
    .named = true,
  },
  [sym_dot] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_namespace_path_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_namespace_body_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_block_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_body = 1,
  field_name = 2,
  field_path = 3,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_body] = "body",
  [field_name] = "name",
  [field_path] = "path",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 1},
  [3] = {.index = 2, .length = 2},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_path, 1},
  [1] =
    {field_body, 1},
  [2] =
    {field_body, 2},
    {field_name, 1},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 4,
  [7] = 7,
  [8] = 5,
  [9] = 9,
  [10] = 10,
  [11] = 9,
  [12] = 12,
  [13] = 10,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 14,
  [23] = 23,
  [24] = 17,
  [25] = 16,
  [26] = 18,
  [27] = 27,
  [28] = 28,
  [29] = 28,
  [30] = 20,
  [31] = 23,
  [32] = 19,
  [33] = 27,
  [34] = 34,
  [35] = 34,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 39,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(11);
      if (lookahead == '"') ADVANCE(1);
      if (lookahead == '\'') ADVANCE(2);
      if (lookahead == '.') ADVANCE(13);
      if (lookahead == '/') ADVANCE(17);
      if (lookahead == ';') ADVANCE(12);
      if (lookahead == '{') ADVANCE(14);
      if (lookahead == '}') ADVANCE(15);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(20);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(19);
      if ((0x80 <= lookahead && lookahead <= 0xffff)) ADVANCE(18);
      if (lookahead != 0) ADVANCE(16);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(21);
      if (lookahead == '\\') ADVANCE(8);
      if (lookahead != 0) ADVANCE(1);
      END_STATE();
    case 2:
      if (lookahead == '\'') ADVANCE(22);
      if (lookahead == '\\') ADVANCE(9);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(23);
      if (lookahead != 0) ADVANCE(3);
      END_STATE();
    case 3:
      if (lookahead == '\'') ADVANCE(22);
      if (lookahead == '\\') ADVANCE(9);
      if (lookahead != 0) ADVANCE(3);
      END_STATE();
    case 4:
      if (lookahead == '*') ADVANCE(6);
      if (lookahead == '/') ADVANCE(24);
      END_STATE();
    case 5:
      if (lookahead == '*') ADVANCE(5);
      if (lookahead == '/') ADVANCE(25);
      if (lookahead != 0) ADVANCE(6);
      END_STATE();
    case 6:
      if (lookahead == '*') ADVANCE(5);
      if (lookahead != 0) ADVANCE(6);
      END_STATE();
    case 7:
      if (lookahead == '/') ADVANCE(4);
      if (lookahead == '{') ADVANCE(14);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(7);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0x80 <= lookahead && lookahead <= 0xffff)) ADVANCE(19);
      END_STATE();
    case 8:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(1);
      END_STATE();
    case 9:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(3);
      END_STATE();
    case 10:
      if (eof) ADVANCE(11);
      if (lookahead == '"') ADVANCE(1);
      if (lookahead == '\'') ADVANCE(2);
      if (lookahead == '.') ADVANCE(13);
      if (lookahead == '/') ADVANCE(17);
      if (lookahead == '{') ADVANCE(14);
      if (lookahead == '}') ADVANCE(15);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(10);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(20);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(19);
      if ((0x80 <= lookahead && lookahead <= 0xffff)) ADVANCE(18);
      if (lookahead != 0) ADVANCE(16);
      END_STATE();
    case 11:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 12:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 13:
      ACCEPT_TOKEN(anon_sym_DOT);
      END_STATE();
    case 14:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 15:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 16:
      ACCEPT_TOKEN(sym_punct);
      END_STATE();
    case 17:
      ACCEPT_TOKEN(sym_punct);
      if (lookahead == '*') ADVANCE(6);
      if (lookahead == '/') ADVANCE(24);
      END_STATE();
    case 18:
      ACCEPT_TOKEN(sym_punct);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0x80 <= lookahead && lookahead <= 0xffff)) ADVANCE(19);
      END_STATE();
    case 19:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0x80 <= lookahead && lookahead <= 0xffff)) ADVANCE(19);
      END_STATE();
    case 20:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(20);
      END_STATE();
    case 21:
      ACCEPT_TOKEN(sym_string);
      END_STATE();
    case 22:
      ACCEPT_TOKEN(sym_char);
      END_STATE();
    case 23:
      ACCEPT_TOKEN(sym_lifetime);
      if (lookahead == '\'') ADVANCE(22);
      if (lookahead == '\\') ADVANCE(9);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(23);
      if (lookahead != 0) ADVANCE(3);
      END_STATE();
    case 24:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(24);
      END_STATE();
    case 25:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    default:
      return false;
  }
}

static bool ts_lex_keywords(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (lookahead == 'i') ADVANCE(1);
      if (lookahead == 'n') ADVANCE(2);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      END_STATE();
    case 1:
      if (lookahead == 'n') ADVANCE(3);
      END_STATE();
    case 2:
      if (lookahead == 'a') ADVANCE(4);
      END_STATE();
    case 3:
      if (lookahead == 'c') ADVANCE(5);
      END_STATE();
    case 4:
      if (lookahead == 'm') ADVANCE(6);
      END_STATE();
    case 5:
      if (lookahead == 'l') ADVANCE(7);
      END_STATE();
    case 6:
      if (lookahead == 'e') ADVANCE(8);
      END_STATE();
    case 7:
      if (lookahead == 'u') ADVANCE(9);
      END_STATE();
    case 8:
      if (lookahead == 's') ADVANCE(10);
      END_STATE();
    case 9:
      if (lookahead == 'd') ADVANCE(11);
      END_STATE();
    case 10:
      if (lookahead == 'p') ADVANCE(12);
      END_STATE();
    case 11:
      if (lookahead == 'e') ADVANCE(13);
      END_STATE();
    case 12:
      if (lookahead == 'a') ADVANCE(14);
      END_STATE();
    case 13:
      ACCEPT_TOKEN(anon_sym_include);
      END_STATE();
    case 14:
      if (lookahead == 'c') ADVANCE(15);
      END_STATE();
    case 15:
      if (lookahead == 'e') ADVANCE(16);
      END_STATE();
    case 16:
      ACCEPT_TOKEN(anon_sym_namespace);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 10},
  [2] = {.lex_state = 10},
  [3] = {.lex_state = 10},
  [4] = {.lex_state = 10},
  [5] = {.lex_state = 10},
  [6] = {.lex_state = 10},
  [7] = {.lex_state = 10},
  [8] = {.lex_state = 10},
  [9] = {.lex_state = 10},
  [10] = {.lex_state = 10},
  [11] = {.lex_state = 10},
  [12] = {.lex_state = 10},
  [13] = {.lex_state = 10},
  [14] = {.lex_state = 0},
  [15] = {.lex_state = 0},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 0},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 10},
  [20] = {.lex_state = 10},
  [21] = {.lex_state = 10},
  [22] = {.lex_state = 0},
  [23] = {.lex_state = 10},
  [24] = {.lex_state = 0},
  [25] = {.lex_state = 0},
  [26] = {.lex_state = 0},
  [27] = {.lex_state = 10},
  [28] = {.lex_state = 10},
  [29] = {.lex_state = 10},
  [30] = {.lex_state = 10},
  [31] = {.lex_state = 10},
  [32] = {.lex_state = 10},
  [33] = {.lex_state = 10},
  [34] = {.lex_state = 7},
  [35] = {.lex_state = 7},
  [36] = {.lex_state = 0},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 0},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 7},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 0},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_identifier] = ACTIONS(1),
    [anon_sym_include] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_namespace] = ACTIONS(1),
    [anon_sym_DOT] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [sym_punct] = ACTIONS(1),
    [sym_number] = ACTIONS(1),
    [sym_string] = ACTIONS(1),
    [sym_char] = ACTIONS(1),
    [sym_lifetime] = ACTIONS(1),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [1] = {
    [sym_source_file] = STATE(43),
    [sym__top] = STATE(3),
    [sym_include] = STATE(3),
    [sym_namespace] = STATE(3),
    [sym__token] = STATE(3),
    [sym_block] = STATE(3),
    [sym_dot] = STATE(3),
    [aux_sym_source_file_repeat1] = STATE(3),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [anon_sym_include] = ACTIONS(9),
    [anon_sym_namespace] = ACTIONS(11),
    [anon_sym_DOT] = ACTIONS(13),
    [anon_sym_LBRACE] = ACTIONS(15),
    [sym_punct] = ACTIONS(7),
    [sym_number] = ACTIONS(17),
    [sym_string] = ACTIONS(17),
    [sym_char] = ACTIONS(17),
    [sym_lifetime] = ACTIONS(7),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [2] = {
    [sym__top] = STATE(2),
    [sym_include] = STATE(2),
    [sym_namespace] = STATE(2),
    [sym__token] = STATE(2),
    [sym_block] = STATE(2),
    [sym_dot] = STATE(2),
    [aux_sym_source_file_repeat1] = STATE(2),
    [ts_builtin_sym_end] = ACTIONS(19),
    [sym_identifier] = ACTIONS(21),
    [anon_sym_include] = ACTIONS(24),
    [anon_sym_namespace] = ACTIONS(27),
    [anon_sym_DOT] = ACTIONS(30),
    [anon_sym_LBRACE] = ACTIONS(33),
    [sym_punct] = ACTIONS(21),
    [sym_number] = ACTIONS(36),
    [sym_string] = ACTIONS(36),
    [sym_char] = ACTIONS(36),
    [sym_lifetime] = ACTIONS(21),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [3] = {
    [sym__top] = STATE(2),
    [sym_include] = STATE(2),
    [sym_namespace] = STATE(2),
    [sym__token] = STATE(2),
    [sym_block] = STATE(2),
    [sym_dot] = STATE(2),
    [aux_sym_source_file_repeat1] = STATE(2),
    [ts_builtin_sym_end] = ACTIONS(39),
    [sym_identifier] = ACTIONS(41),
    [anon_sym_include] = ACTIONS(9),
    [anon_sym_namespace] = ACTIONS(11),
    [anon_sym_DOT] = ACTIONS(13),
    [anon_sym_LBRACE] = ACTIONS(15),
    [sym_punct] = ACTIONS(41),
    [sym_number] = ACTIONS(43),
    [sym_string] = ACTIONS(43),
    [sym_char] = ACTIONS(43),
    [sym_lifetime] = ACTIONS(41),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [4] = {
    [sym_namespace] = STATE(5),
    [sym__token] = STATE(5),
    [sym_block] = STATE(5),
    [sym_dot] = STATE(5),
    [aux_sym_namespace_body_repeat1] = STATE(5),
    [sym_identifier] = ACTIONS(45),
    [anon_sym_namespace] = ACTIONS(47),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(53),
    [sym_punct] = ACTIONS(45),
    [sym_number] = ACTIONS(55),
    [sym_string] = ACTIONS(55),
    [sym_char] = ACTIONS(55),
    [sym_lifetime] = ACTIONS(45),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [5] = {
    [sym_namespace] = STATE(7),
    [sym__token] = STATE(7),
    [sym_block] = STATE(7),
    [sym_dot] = STATE(7),
    [aux_sym_namespace_body_repeat1] = STATE(7),
    [sym_identifier] = ACTIONS(57),
    [anon_sym_namespace] = ACTIONS(47),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(59),
    [sym_punct] = ACTIONS(57),
    [sym_number] = ACTIONS(61),
    [sym_string] = ACTIONS(61),
    [sym_char] = ACTIONS(61),
    [sym_lifetime] = ACTIONS(57),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [6] = {
    [sym_namespace] = STATE(8),
    [sym__token] = STATE(8),
    [sym_block] = STATE(8),
    [sym_dot] = STATE(8),
    [aux_sym_namespace_body_repeat1] = STATE(8),
    [sym_identifier] = ACTIONS(63),
    [anon_sym_namespace] = ACTIONS(47),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(65),
    [sym_punct] = ACTIONS(63),
    [sym_number] = ACTIONS(67),
    [sym_string] = ACTIONS(67),
    [sym_char] = ACTIONS(67),
    [sym_lifetime] = ACTIONS(63),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [7] = {
    [sym_namespace] = STATE(7),
    [sym__token] = STATE(7),
    [sym_block] = STATE(7),
    [sym_dot] = STATE(7),
    [aux_sym_namespace_body_repeat1] = STATE(7),
    [sym_identifier] = ACTIONS(69),
    [anon_sym_namespace] = ACTIONS(72),
    [anon_sym_DOT] = ACTIONS(75),
    [anon_sym_LBRACE] = ACTIONS(78),
    [anon_sym_RBRACE] = ACTIONS(81),
    [sym_punct] = ACTIONS(69),
    [sym_number] = ACTIONS(83),
    [sym_string] = ACTIONS(83),
    [sym_char] = ACTIONS(83),
    [sym_lifetime] = ACTIONS(69),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [8] = {
    [sym_namespace] = STATE(7),
    [sym__token] = STATE(7),
    [sym_block] = STATE(7),
    [sym_dot] = STATE(7),
    [aux_sym_namespace_body_repeat1] = STATE(7),
    [sym_identifier] = ACTIONS(57),
    [anon_sym_namespace] = ACTIONS(47),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(86),
    [sym_punct] = ACTIONS(57),
    [sym_number] = ACTIONS(61),
    [sym_string] = ACTIONS(61),
    [sym_char] = ACTIONS(61),
    [sym_lifetime] = ACTIONS(57),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [9] = {
    [sym__token] = STATE(10),
    [sym_block] = STATE(10),
    [sym_dot] = STATE(10),
    [aux_sym_block_repeat1] = STATE(10),
    [sym_identifier] = ACTIONS(88),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(90),
    [sym_punct] = ACTIONS(88),
    [sym_number] = ACTIONS(92),
    [sym_string] = ACTIONS(92),
    [sym_char] = ACTIONS(92),
    [sym_lifetime] = ACTIONS(88),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [10] = {
    [sym__token] = STATE(12),
    [sym_block] = STATE(12),
    [sym_dot] = STATE(12),
    [aux_sym_block_repeat1] = STATE(12),
    [sym_identifier] = ACTIONS(94),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(96),
    [sym_punct] = ACTIONS(94),
    [sym_number] = ACTIONS(98),
    [sym_string] = ACTIONS(98),
    [sym_char] = ACTIONS(98),
    [sym_lifetime] = ACTIONS(94),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [11] = {
    [sym__token] = STATE(13),
    [sym_block] = STATE(13),
    [sym_dot] = STATE(13),
    [aux_sym_block_repeat1] = STATE(13),
    [sym_identifier] = ACTIONS(100),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(102),
    [sym_punct] = ACTIONS(100),
    [sym_number] = ACTIONS(104),
    [sym_string] = ACTIONS(104),
    [sym_char] = ACTIONS(104),
    [sym_lifetime] = ACTIONS(100),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [12] = {
    [sym__token] = STATE(12),
    [sym_block] = STATE(12),
    [sym_dot] = STATE(12),
    [aux_sym_block_repeat1] = STATE(12),
    [sym_identifier] = ACTIONS(106),
    [anon_sym_DOT] = ACTIONS(109),
    [anon_sym_LBRACE] = ACTIONS(112),
    [anon_sym_RBRACE] = ACTIONS(115),
    [sym_punct] = ACTIONS(106),
    [sym_number] = ACTIONS(117),
    [sym_string] = ACTIONS(117),
    [sym_char] = ACTIONS(117),
    [sym_lifetime] = ACTIONS(106),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [13] = {
    [sym__token] = STATE(12),
    [sym_block] = STATE(12),
    [sym_dot] = STATE(12),
    [aux_sym_block_repeat1] = STATE(12),
    [sym_identifier] = ACTIONS(94),
    [anon_sym_DOT] = ACTIONS(49),
    [anon_sym_LBRACE] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(120),
    [sym_punct] = ACTIONS(94),
    [sym_number] = ACTIONS(98),
    [sym_string] = ACTIONS(98),
    [sym_char] = ACTIONS(98),
    [sym_lifetime] = ACTIONS(94),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 4,
    ACTIONS(126), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(124), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(122), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [23] = 4,
    ACTIONS(132), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(130), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(128), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [46] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(136), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(134), 7,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [67] = 4,
    ACTIONS(142), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(140), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(138), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [90] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(146), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(144), 7,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [111] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(150), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(148), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [131] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(154), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(152), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [151] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(158), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(156), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [171] = 4,
    ACTIONS(160), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(124), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(122), 6,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [193] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(164), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(162), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [213] = 4,
    ACTIONS(166), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(140), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(138), 6,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [235] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(136), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(134), 7,
      anon_sym_SEMI,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [255] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(146), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(144), 7,
      anon_sym_SEMI,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [275] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(170), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(168), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [295] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(174), 5,
      anon_sym_include,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(172), 6,
      ts_builtin_sym_end,
      anon_sym_DOT,
      anon_sym_LBRACE,
      sym_number,
      sym_string,
      sym_char,
  [315] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(174), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(172), 6,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [334] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(154), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(152), 6,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [353] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(164), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(162), 6,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [372] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(150), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(148), 6,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [391] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(170), 4,
      anon_sym_namespace,
      sym_punct,
      sym_identifier,
      sym_lifetime,
    ACTIONS(168), 6,
      anon_sym_DOT,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_number,
      sym_string,
      sym_char,
  [410] = 5,
    ACTIONS(176), 1,
      sym_identifier,
    ACTIONS(178), 1,
      anon_sym_LBRACE,
    STATE(14), 1,
      sym_namespace_body,
    STATE(40), 1,
      sym_namespace_path,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [427] = 5,
    ACTIONS(176), 1,
      sym_identifier,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    STATE(22), 1,
      sym_namespace_body,
    STATE(39), 1,
      sym_namespace_path,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [444] = 4,
    ACTIONS(182), 1,
      anon_sym_DOT,
    ACTIONS(184), 1,
      anon_sym_LBRACE,
    STATE(38), 1,
      aux_sym_namespace_path_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [458] = 4,
    ACTIONS(186), 1,
      anon_sym_DOT,
    ACTIONS(189), 1,
      anon_sym_LBRACE,
    STATE(37), 1,
      aux_sym_namespace_path_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [472] = 4,
    ACTIONS(182), 1,
      anon_sym_DOT,
    ACTIONS(191), 1,
      anon_sym_LBRACE,
    STATE(37), 1,
      aux_sym_namespace_path_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [486] = 3,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    STATE(24), 1,
      sym_namespace_body,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [497] = 3,
    ACTIONS(178), 1,
      anon_sym_LBRACE,
    STATE(17), 1,
      sym_namespace_body,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [508] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(189), 2,
      anon_sym_DOT,
      anon_sym_LBRACE,
  [517] = 2,
    ACTIONS(193), 1,
      sym_identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [525] = 2,
    ACTIONS(195), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [533] = 2,
    ACTIONS(197), 1,
      sym_string,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(14)] = 0,
  [SMALL_STATE(15)] = 23,
  [SMALL_STATE(16)] = 46,
  [SMALL_STATE(17)] = 67,
  [SMALL_STATE(18)] = 90,
  [SMALL_STATE(19)] = 111,
  [SMALL_STATE(20)] = 131,
  [SMALL_STATE(21)] = 151,
  [SMALL_STATE(22)] = 171,
  [SMALL_STATE(23)] = 193,
  [SMALL_STATE(24)] = 213,
  [SMALL_STATE(25)] = 235,
  [SMALL_STATE(26)] = 255,
  [SMALL_STATE(27)] = 275,
  [SMALL_STATE(28)] = 295,
  [SMALL_STATE(29)] = 315,
  [SMALL_STATE(30)] = 334,
  [SMALL_STATE(31)] = 353,
  [SMALL_STATE(32)] = 372,
  [SMALL_STATE(33)] = 391,
  [SMALL_STATE(34)] = 410,
  [SMALL_STATE(35)] = 427,
  [SMALL_STATE(36)] = 444,
  [SMALL_STATE(37)] = 458,
  [SMALL_STATE(38)] = 472,
  [SMALL_STATE(39)] = 486,
  [SMALL_STATE(40)] = 497,
  [SMALL_STATE(41)] = 508,
  [SMALL_STATE(42)] = 517,
  [SMALL_STATE(43)] = 525,
  [SMALL_STATE(44)] = 533,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(44),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [19] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [21] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(2),
  [24] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(44),
  [27] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(34),
  [30] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [33] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(9),
  [36] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(2),
  [39] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [49] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [51] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [53] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [55] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [57] = {.entry = {.count = 1, .reusable = false}}, SHIFT(7),
  [59] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [61] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [63] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [65] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [67] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [69] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_namespace_body_repeat1, 2, 0, 0), SHIFT_REPEAT(7),
  [72] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_namespace_body_repeat1, 2, 0, 0), SHIFT_REPEAT(35),
  [75] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_namespace_body_repeat1, 2, 0, 0), SHIFT_REPEAT(30),
  [78] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_namespace_body_repeat1, 2, 0, 0), SHIFT_REPEAT(11),
  [81] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_namespace_body_repeat1, 2, 0, 0),
  [83] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_namespace_body_repeat1, 2, 0, 0), SHIFT_REPEAT(7),
  [86] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [88] = {.entry = {.count = 1, .reusable = false}}, SHIFT(10),
  [90] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [92] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [94] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [96] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [98] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [100] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [102] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [104] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [106] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(12),
  [109] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(30),
  [112] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(11),
  [115] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [117] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(12),
  [120] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [122] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace, 2, 0, 2),
  [124] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_namespace, 2, 0, 2),
  [126] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [128] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_include, 2, 0, 1),
  [130] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_include, 2, 0, 1),
  [132] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [134] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace_body, 2, 0, 0),
  [136] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_namespace_body, 2, 0, 0),
  [138] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace, 3, 0, 3),
  [140] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_namespace, 3, 0, 3),
  [142] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [144] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace_body, 3, 0, 0),
  [146] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_namespace_body, 3, 0, 0),
  [148] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [150] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_block, 2, 0, 0),
  [152] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_dot, 1, 0, 0),
  [154] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_dot, 1, 0, 0),
  [156] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_include, 3, 0, 1),
  [158] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_include, 3, 0, 1),
  [160] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [162] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [164] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_block, 3, 0, 0),
  [166] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [168] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace, 4, 0, 3),
  [170] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_namespace, 4, 0, 3),
  [172] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace, 3, 0, 2),
  [174] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_namespace, 3, 0, 2),
  [176] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [178] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [180] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [182] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [184] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace_path, 1, 0, 0),
  [186] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_namespace_path_repeat1, 2, 0, 0), SHIFT_REPEAT(42),
  [189] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_namespace_path_repeat1, 2, 0, 0),
  [191] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace_path, 2, 0, 0),
  [193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [195] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_lrs(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .keyword_lex_fn = ts_lex_keywords,
    .keyword_capture_token = sym_identifier,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
