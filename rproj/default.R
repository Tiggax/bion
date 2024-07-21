library(ggplot2)
library(tidyr)
library(dplyr)
library(readr)

setwd("/mnt/Working/Programing/Git/tiggax/bion/data")


simple_fit <- read.csv("default.csv")

simple_fit %>% 
  #mutate(DO = DO * 10000) %>% 
  pivot_longer(cols = !minutes, names_to = "type", values_to = "val") %>% 
  filter(type != "product")-> data

data %>% 
  ggplot(aes(x = minutes, y = val, colour = type)) + 
  geom_line() 

dir.create("fig")

ggsave()