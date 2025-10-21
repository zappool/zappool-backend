from html.parser import HTMLParser
from html.entities import name2codepoint

def get_class(attrs):
    for attr in attrs:
        if len(attr) >= 2:
            if attr[0] == "class":
                return attr[1]
    return ""

class MyHTMLParser(HTMLParser):
    values = {}

    current_tag_stack: list[str] = []
    level: int = 0
    in_dash_block: bool = False
    in_dash_block_level: int = 0
    in_label: bool = False
    in_label_level: int = 0
    label: str = ""
    in_value: bool = False
    in_value_level: int = 0
    value: str = ""

    def handle_starttag(self, tag, attrs):
        # for i in range(0, self.level):
        #     print("  ", end="")
        # print(tag)

        if tag == "div":
            cl = get_class(attrs)
            if "dashboard-container" in cl:
                self.in_dash_block = True
                self.in_dash_block_level = self.level
                # print(f"== Opened dashboard block, class {cl}, {self.level}")
            elif (self.in_dash_block) and (self.level == self.in_dash_block_level + 1) and ("label" in cl):
                    self.in_label = True
                    self.in_label_level = self.level
                    self.label = ""
                    # print(f"== Opened label, class {cl}, {self.level}")
        if tag == "span":
            if (self.in_dash_block) and (self.level == self.in_dash_block_level + 1):
                self.in_value = True
                self.in_value_level = self.level
                self.value = ""
                # print(f"== Opened value, {self.level}")

        self.current_tag_stack.append(tag)
        self.level += 1

    def handle_endtag(self, tag):
        if len(self.current_tag_stack) > 0:
            last_tag = self.current_tag_stack[len(self.current_tag_stack) - 1]
            if tag != last_tag:
                self.handle_endtag(last_tag)
        del self.current_tag_stack[-1]
        self.level = max(self.level - 1, 0)

        # for i in range(0, self.level):
        #     print("  ", end="")
        # print(f"/{tag}")

        # print("End tag  :", tag)
        if tag == "div":
            if self.in_dash_block and (self.level == self.in_dash_block_level):
                # print(f"==== Closed dashboard block  Label '{self.label}'  Value '{self.value}'  {self.level}")
                self.values[self.label] = self.value

                self.in_dash_block = False
                self.in_dash_block_level = 0
                self.in_label = False
                self.in_label_level = 0
                self.label = ""
                self.in_value = False
                self.in_value_level = 0
                self.value = ""
            elif self.in_label and (self.level == self.in_label_level):
                self.in_label = False
                self.in_label_level = 0
                # print(f"== Closed level {self.level}")
        if tag == "span":
            if (self.in_value) and (self.level == self.in_value_level):
                self.in_value = False
                self.in_value_level = 0
                # print(f"== Closed value {self.level}")


    def handle_data(self, data):
        data_strip = data.strip()
        if self.in_label and (self.level == self.in_label_level + 1) and (len(data_strip) > 0):
            # print(self.in_label, self.level, self.in_label_level, len(data))
            self.label = data_strip
            # print(f"== Label: '{self.label}'")
        if self.in_value and (self.level == self.in_value_level + 1) and (len(data_strip) > 0):
            self.value = data_strip
            # print(f"== Value: '{self.value}'")

    # def handle_comment(self, data):
    #     print("Comment  :", data)

    # def handle_entityref(self, name):
    #     c = chr(name2codepoint[name])
    #     print("Named ent:", c)

    # def handle_charref(self, name):
    #     if name.startswith('x'):
    #         c = chr(int(name[1:], 16))
    #     else:
    #         c = chr(int(name))
    #     print("Num ent  :", c)

    # def handle_decl(self, data):
    #     print("Decl     :", data)

def key_value_pairs_from_html(html: str) -> dict[str, str]:
    parser = MyHTMLParser()
    parser.feed(html)
    return parser.values






# with open('stat2.html') as f:
#     lines = f.readlines()
#     s = ""
#     for l in lines:
#         s += l

# values = key_value_pairs_from_html(s)
# print(f"Found {len(values)} key-value pairs:")
# for k in values:
#     print(f"'{k}': '{values[k]}'")