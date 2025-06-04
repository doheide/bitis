//
// Created by dheide on 04.06.25.
//
template <typename T>
int write_or_test(const char *fn_name, T &msg, const char *arg) {
    auto repl_str = std::string(".cpp.");

    if(arg==nullptr) {
        auto bin_msg = serialize(msg);

        std::ofstream outfile{fn_name, std::ios::binary};
        outfile.write(reinterpret_cast<const char *>(bin_msg.data()), bin_msg.size());
        outfile.close();
    }
    else {
        auto fn_mod = std::string(fn_name);
        auto new_str = "." + std::string(arg) + ".";
        fn_mod.replace(fn_mod.find(repl_str), repl_str.length(), new_str);

        printf("loading: %s\n", fn_mod.c_str());

        std::ifstream infile(fn_mod, std::ifstream::binary);

        infile.seekg(0, infile.end);     //N is the total number of doubles
        auto file_size = infile.tellg();
        infile.seekg(0, infile.beg);

        std::vector<uint8_t> bin_data(file_size);
        infile.read(reinterpret_cast<char *>(bin_data.data()), file_size);
        infile.close();

        auto bin_msg = deserialize<T>(bin_data);
        printf("deserialized:\n"); bin_msg.print();printf("\n");
        if (msg == bin_msg) {
            printf("* OK\n");
        }
        else {
            printf("* FAILED\n");
            return 1;
        }
    }
    return 0;
}
